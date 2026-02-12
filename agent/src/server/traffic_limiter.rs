use anyhow::Result;
use chrono::{Datelike, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use quinn::VarInt;

use crate::server::entity::{client, user, user_client, Client, User, UserClient};
use crate::server::proxy_server::ProxyListenerManager;

/// GB 转字节
pub fn gb_to_bytes(gb: f64) -> i64 {
    (gb * 1024.0 * 1024.0 * 1024.0) as i64
}

/// 字节转 GB
pub fn bytes_to_gb(bytes: i64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}

/// 判断是否需要重置流量
pub fn should_reset_traffic(user: &user::Model) -> bool {
    let now = Utc::now().naive_utc();

    if user.last_reset_at.is_none() {
        return true; // 从未重置过，需要初始化
    }

    let last_reset = user.last_reset_at.unwrap();

    match user.traffic_reset_cycle.as_str() {
        "daily" => {
            // 检查日期是否不同
            now.date() > last_reset.date()
        },
        "monthly" => {
            // 检查月份是否不同
            now.year() > last_reset.year() ||
            (now.year() == last_reset.year() && now.month() > last_reset.month())
        },
        _ => false, // "none" 或其他值，不重置
    }
}

/// 重置用户流量统计
pub async fn reset_user_traffic(user_id: i64, db: &DatabaseConnection) -> Result<()> {
    let user = match User::find_by_id(user_id).one(db).await? {
        Some(u) => u,
        None => return Ok(()),
    };

    let mut user_active: user::ActiveModel = user.into();
    user_active.total_bytes_sent = Set(0);
    user_active.total_bytes_received = Set(0);
    user_active.is_traffic_exceeded = Set(false);
    user_active.last_reset_at = Set(Some(Utc::now().naive_utc()));
    user_active.updated_at = Set(Utc::now().naive_utc());

    user_active.update(db).await?;
    info!("✅ 用户 #{} 流量已重置", user_id);

    Ok(())
}

/// 检查用户流量是否超限
/// 返回 (是否超限, 超限原因)
pub async fn check_user_traffic_limit(user_id: i64, db: &DatabaseConnection) -> Result<(bool, String)> {
    let user = match User::find_by_id(user_id).one(db).await? {
        Some(u) => u,
        None => return Ok((false, String::new())),
    };

    // 检查是否需要重置流量
    if should_reset_traffic(&user) {
        reset_user_traffic(user_id, db).await?;
        return Ok((false, String::new()));
    }

    // 检查上传流量限制
    if let Some(upload_limit_gb) = user.upload_limit_gb {
        let upload_limit_bytes = gb_to_bytes(upload_limit_gb);
        if user.total_bytes_sent >= upload_limit_bytes {
            let reason = format!(
                "上传流量超限: {:.2} GB / {:.2} GB",
                bytes_to_gb(user.total_bytes_sent),
                upload_limit_gb
            );
            return Ok((true, reason));
        }
    }

    // 检查下载流量限制
    if let Some(download_limit_gb) = user.download_limit_gb {
        let download_limit_bytes = gb_to_bytes(download_limit_gb);
        if user.total_bytes_received >= download_limit_bytes {
            let reason = format!(
                "下载流量超限: {:.2} GB / {:.2} GB",
                bytes_to_gb(user.total_bytes_received),
                download_limit_gb
            );
            return Ok((true, reason));
        }
    }

    Ok((false, String::new()))
}

/// 断开用户的所有客户端连接
pub async fn disconnect_user_clients(
    user_id: i64,
    connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    listener_manager: Arc<ProxyListenerManager>,
    db: &DatabaseConnection,
) -> Result<()> {
    // 1. 查询该用户的所有客户端
    let user_clients = UserClient::find()
        .filter(user_client::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    if user_clients.is_empty() {
        return Ok(());
    }

    info!("🚫 用户 #{} 流量超限，正在断开 {} 个客户端连接", user_id, user_clients.len());

    // 2. 停止每个客户端的代理监听器并断开连接
    for uc in user_clients {
        let client_id_str = format!("{}", uc.client_id);

        // 停止代理监听器
        listener_manager.stop_client_proxies(&client_id_str).await;

        // 断开 QUIC 连接
        let mut conns = connections.write().await;
        if let Some(conn) = conns.remove(&client_id_str) {
            conn.close(VarInt::from_u32(1), b"traffic limit exceeded");
            warn!("  断开客户端 #{} 的连接：流量超限", uc.client_id);
        }
        drop(conns);

        // 更新客户端离线状态
        if let Some(client) = Client::find_by_id(uc.client_id).one(db).await? {
            let mut client_active: client::ActiveModel = client.into();
            client_active.is_online = Set(false);
            client_active.updated_at = Set(Utc::now().naive_utc());
            if let Err(e) = client_active.update(db).await {
                error!("更新客户端 #{} 离线状态失败: {}", uc.client_id, e);
            }
        }
    }

    Ok(())
}

/// 检查并处理用户流量超限（在流量统计更新后调用）
pub async fn check_and_handle_traffic_exceeded(
    user_id: i64,
    connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    listener_manager: Arc<ProxyListenerManager>,
    db: &DatabaseConnection,
) -> Result<()> {
    let (is_exceeded, reason) = check_user_traffic_limit(user_id, db).await?;

    if is_exceeded {
        // 获取用户当前状态
        let user = match User::find_by_id(user_id).one(db).await? {
            Some(u) => u,
            None => return Ok(()),
        };

        // 如果之前未标记为超限，则现在标记并断开连接
        if !user.is_traffic_exceeded {
            warn!("⚠️ 用户 #{} ({}): {}", user_id, user.username, reason);

            // 更新超限状态
            let mut user_active: user::ActiveModel = user.into();
            user_active.is_traffic_exceeded = Set(true);
            user_active.updated_at = Set(Utc::now().naive_utc());
            user_active.update(db).await?;

            // 断开该用户的所有客户端
            disconnect_user_clients(user_id, connections, listener_manager, db).await?;
        }
    }

    Ok(())
}

// ============== 节点（Client）级别流量限制 ==============

/// 判断节点是否需要重置流量
pub fn should_reset_client_traffic(client: &client::Model) -> bool {
    let now = Utc::now().naive_utc();

    if client.last_reset_at.is_none() {
        return true; // 从未重置过，需要初始化
    }

    let last_reset = client.last_reset_at.unwrap();

    match client.traffic_reset_cycle.as_str() {
        "daily" => {
            // 检查日期是否不同
            now.date() > last_reset.date()
        },
        "monthly" => {
            // 检查月份是否不同
            now.year() > last_reset.year() ||
            (now.year() == last_reset.year() && now.month() > last_reset.month())
        },
        _ => false, // "none" 或其他值，不重置
    }
}

/// 重置节点流量统计
pub async fn reset_client_traffic(client_id: i64, db: &DatabaseConnection) -> Result<()> {
    let client_model = match Client::find_by_id(client_id).one(db).await? {
        Some(c) => c,
        None => return Ok(()),
    };

    let mut client_active: client::ActiveModel = client_model.into();
    client_active.total_bytes_sent = Set(0);
    client_active.total_bytes_received = Set(0);
    client_active.is_traffic_exceeded = Set(false);
    client_active.last_reset_at = Set(Some(Utc::now().naive_utc()));
    client_active.updated_at = Set(Utc::now().naive_utc());

    client_active.update(db).await?;
    info!("✅ 节点 #{} 流量已重置", client_id);

    Ok(())
}

/// 检查节点流量是否超限
/// 返回 (是否超限, 超限原因)
pub async fn check_client_traffic_limit(client_id: i64, db: &DatabaseConnection) -> Result<(bool, String)> {
    let client_model = match Client::find_by_id(client_id).one(db).await? {
        Some(c) => c,
        None => return Ok((false, String::new())),
    };

    // 检查是否需要重置流量
    if should_reset_client_traffic(&client_model) {
        reset_client_traffic(client_id, db).await?;
        return Ok((false, String::new()));
    }

    // 检查上传流量限制
    if let Some(upload_limit_gb) = client_model.upload_limit_gb {
        let upload_limit_bytes = gb_to_bytes(upload_limit_gb);
        if client_model.total_bytes_sent >= upload_limit_bytes {
            let reason = format!(
                "上传流量超限: {:.2} GB / {:.2} GB",
                bytes_to_gb(client_model.total_bytes_sent),
                upload_limit_gb
            );
            return Ok((true, reason));
        }
    }

    // 检查下载流量限制
    if let Some(download_limit_gb) = client_model.download_limit_gb {
        let download_limit_bytes = gb_to_bytes(download_limit_gb);
        if client_model.total_bytes_received >= download_limit_bytes {
            let reason = format!(
                "下载流量超限: {:.2} GB / {:.2} GB",
                bytes_to_gb(client_model.total_bytes_received),
                download_limit_gb
            );
            return Ok((true, reason));
        }
    }

    Ok((false, String::new()))
}

/// 断开单个节点的连接
pub async fn disconnect_client(
    client_id: i64,
    connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    listener_manager: Arc<ProxyListenerManager>,
    db: &DatabaseConnection,
) -> Result<()> {
    let client_id_str = format!("{}", client_id);

    info!("🚫 节点 #{} 流量超限，正在断开连接", client_id);

    // 停止代理监听器
    listener_manager.stop_client_proxies(&client_id_str).await;

    // 断开 QUIC 连接
    let mut conns = connections.write().await;
    if let Some(conn) = conns.remove(&client_id_str) {
        conn.close(VarInt::from_u32(1), b"traffic limit exceeded");
        warn!("  断开节点 #{} 的连接：流量超限", client_id);
    }
    drop(conns);

    // 更新节点离线状态和超限状态
    if let Some(client_model) = Client::find_by_id(client_id).one(db).await? {
        let mut client_active: client::ActiveModel = client_model.into();
        client_active.is_online = Set(false);
        client_active.is_traffic_exceeded = Set(true);
        client_active.updated_at = Set(Utc::now().naive_utc());
        if let Err(e) = client_active.update(db).await {
            error!("更新节点 #{} 状态失败: {}", client_id, e);
        }
    }

    Ok(())
}
