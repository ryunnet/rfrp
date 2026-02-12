use std::sync::Arc;
use crate::server::{config_manager, proxy_server, traffic, local_proxy_control::LocalProxyControl, local_auth_provider::LocalClientAuthProvider};

mod proxy;
mod api;

pub async fn init_features() {
    // 初始化配置管理器
    let config_manager = Arc::new(config_manager::ConfigManager::new());
    if let Err(e) = config_manager.load_from_db().await {
        tracing::error!("加载系统配置失败: {}", e);
    }
    // 初始化流量管理器
    let traffic_manager = std::sync::Arc::new(traffic::TrafficManager::new());
    // 创建本地认证提供者实例
    let auth_provider = Arc::new(LocalClientAuthProvider::new());

    let proxy_server = Arc::new(proxy_server::ProxyServer::new(traffic_manager.clone(), config_manager.clone(), auth_provider.clone()).unwrap());

    // 创建本地代理控制实例
    let proxy_control = Arc::new(LocalProxyControl::new(
        proxy_server.get_listener_manager(),
        proxy_server.get_client_connections(),
        proxy_server.get_tunnel_connections(),
    ));

    // 在此处初始化和注册各种功能模块
    proxy::enable_proxy_feature(proxy_server.clone(), config_manager.clone()).await;
    api::enable_api_feature(proxy_server.clone(), proxy_control, auth_provider, config_manager).await;
}
