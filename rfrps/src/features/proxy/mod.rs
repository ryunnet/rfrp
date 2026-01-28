use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use std::sync::Arc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tracing::info;
use crate::config::get_config;
use crate::migration::get_connection;
use crate::server::ProxyServer;

pub async fn enable_proxy_feature(proxy_server: Arc<ProxyServer>) {
    let config = get_config().await;
    // 启动 QUIC 代理服务器
    tokio::spawn(async move {
        // 重置所有客户端为离线状态（服务端重启后清理僵尸状态）
        if let Err(e) = reset_all_clients_offline().await {
            tracing::warn!("重置客户端状态失败: {}", e);
        }

        let bind_addr = format!("0.0.0.0:{}", config.bind_port);
        proxy_server.run(bind_addr).await.unwrap();
    });
}

/// 重置所有客户端为离线状态
async fn reset_all_clients_offline() -> anyhow::Result<(), sea_orm::DbErr> {
    use crate::entity::{Client, client};
    let db = get_connection().await;

    // 查询所有在线的客户端
    let online_clients = Client::find()
        .filter(client::Column::IsOnline.eq(true))
        .all(db)
        .await?;

    if !online_clients.is_empty() {
        info!("🔄 服务端重启，重置 {} 个客户端状态为离线", online_clients.len());
        for client in online_clients {
            let mut client_active: client::ActiveModel = client.into();
            client_active.is_online = Set(false);
            if let Err(e) = client_active.update(db).await {
                tracing::error!("Failed to reset client status: {}", e);
            }
        }
    }

    Ok(())
}