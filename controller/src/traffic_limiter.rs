use anyhow::Result;
use chrono::{Datelike, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{info};

use crate::entity::{client, user, Client, User};

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
            now.date() > last_reset.date()
        },
        "monthly" => {
            now.year() > last_reset.year() ||
            (now.year() == last_reset.year() && now.month() > last_reset.month())
        },
        _ => false,
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
pub async fn check_user_traffic_limit(user_id: i64, db: &DatabaseConnection) -> Result<(bool, String)> {
    let user = match User::find_by_id(user_id).one(db).await? {
        Some(u) => u,
        None => return Ok((false, String::new())),
    };

    if should_reset_traffic(&user) {
        reset_user_traffic(user_id, db).await?;
        return Ok((false, String::new()));
    }

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

// ============== 节点（Client）级别流量限制 ==============

/// 判断节点是否需要重置流量
pub fn should_reset_client_traffic(client: &client::Model) -> bool {
    let now = Utc::now().naive_utc();

    if client.last_reset_at.is_none() {
        return true;
    }

    let last_reset = client.last_reset_at.unwrap();

    match client.traffic_reset_cycle.as_str() {
        "daily" => {
            now.date() > last_reset.date()
        },
        "monthly" => {
            now.year() > last_reset.year() ||
            (now.year() == last_reset.year() && now.month() > last_reset.month())
        },
        _ => false,
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
pub async fn check_client_traffic_limit(client_id: i64, db: &DatabaseConnection) -> Result<(bool, String)> {
    let client_model = match Client::find_by_id(client_id).one(db).await? {
        Some(c) => c,
        None => return Ok((false, String::new())),
    };

    if should_reset_client_traffic(&client_model) {
        reset_client_traffic(client_id, db).await?;
        return Ok((false, String::new()));
    }

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
