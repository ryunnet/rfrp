use std::sync::Arc;
use crate::{config_manager, server, traffic};
use crate::server::ProxyServer;

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
    // 创建 ProxyServer 实例
    let proxy_server = Arc::new(server::ProxyServer::new(traffic_manager.clone(), config_manager.clone()).unwrap());

    // 在此处初始化和注册各种功能模块
    proxy::enable_proxy_feature(proxy_server.clone()).await;
    api::enable_api_feature(proxy_server.clone(), config_manager).await;
}