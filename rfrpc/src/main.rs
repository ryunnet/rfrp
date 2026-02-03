mod client;
mod config;
mod log_collector;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*, layer::SubscriberExt};
use log_collector::{LogCollector, LogCollectorLayer};
use config::TunnelProtocol;

// 从共享库导入隧道模块
use rfrp_common::{TunnelConnector, QuicConnector, KcpConnector};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志收集器（保留最近 1000 条日志）
    let log_collector = LogCollector::new(1000);

    // 初始化 tracing 日志系统
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .with(LogCollectorLayer::new(log_collector.clone()))
        .init();

    rustls::crypto::ring::default_provider().install_default().unwrap();

    // 加载配置文件
    let cfg = config::Config::load_default()?;

    info!("Loaded configuration: rfrpc.toml");
    info!("Server address: {}:{}", cfg.server_addr, cfg.server_port);
    info!("Token: {}", cfg.token);
    info!("Protocol: {:?}", cfg.protocol);

    let server_addr = cfg.get_server_addr()?;

    // 根据协议创建对应的连接器
    let connector: Arc<dyn TunnelConnector> = match cfg.protocol {
        TunnelProtocol::Quic => {
            info!("Using QUIC protocol");
            Arc::new(QuicConnector::new()?)
        }
        TunnelProtocol::Kcp => {
            info!("Using KCP protocol");
            Arc::new(KcpConnector::new(cfg.kcp.clone()))
        }
    };

    // 使用连接器和日志收集器运行客户端
    client::run(connector, server_addr, cfg.token, log_collector).await?;

    Ok(())
}
