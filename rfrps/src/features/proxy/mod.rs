use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use std::sync::Arc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tracing::info;
use crate::config::{get_config, KcpConfig};
use crate::config_manager::ConfigManager;
use crate::migration::get_connection;
use crate::server::ProxyServer;

/// 启用代理功能
///
/// 根据配置选择使用 QUIC 或 KCP 协议，二者只能选其一，使用相同的端口
pub async fn enable_proxy_feature(proxy_server: Arc<ProxyServer>, config_manager: Arc<ConfigManager>) {
    let config = get_config().await;
    let bind_addr = format!("0.0.0.0:{}", config.bind_port);

    // 重置所有客户端为离线状态
    if let Err(e) = reset_all_clients_offline().await {
        tracing::warn!("Failed to reset client status: {}", e);
    }

    // 根据配置选择协议：use_kcp = true 使用 KCP，否则使用 QUIC
    let use_kcp = config_manager.get_bool("use_kcp", false).await;

    if use_kcp {
        // 使用 KCP 协议
        let kcp_config = load_kcp_config(&config_manager).await;
        info!("Starting KCP server on {}", bind_addr);

        tokio::spawn(async move {
            if let Err(e) = proxy_server.run_kcp(bind_addr, Some(kcp_config)).await {
                tracing::error!("KCP server error: {}", e);
            }
        });
    } else {
        // 使用 QUIC 协议（默认）
        info!("Starting QUIC server on {}", bind_addr);

        tokio::spawn(async move {
            if let Err(e) = proxy_server.run(bind_addr).await {
                tracing::error!("QUIC server error: {}", e);
            }
        });
    }
}

/// Load KCP configuration from ConfigManager
async fn load_kcp_config(config_manager: &ConfigManager) -> KcpConfig {
    KcpConfig {
        nodelay: config_manager.get_bool("kcp_nodelay", true).await,
        interval: config_manager.get_number("kcp_interval", 10).await as u32,
        resend: config_manager.get_number("kcp_resend", 2).await as u32,
        nc: config_manager.get_bool("kcp_nc", true).await,
    }
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
        info!("Server restart, resetting {} clients to offline", online_clients.len());
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
