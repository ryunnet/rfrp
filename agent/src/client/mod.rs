pub mod connector;
pub mod config;
pub mod log_collector;

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*, layer::SubscriberExt};
use log_collector::{LogCollector, LogCollectorLayer};
use config::TunnelProtocol;

// 从共享库导入隧道模块
use common::{TunnelConnector, QuicConnector, KcpConnector};

pub async fn run_client(
    config_path: Option<String>,
    controller_url: Option<String>,
    token: Option<String>,
) -> Result<()> {
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

    // 判断运行模式
    let is_controller_mode = controller_url.is_some();

    if is_controller_mode {
        let controller_url = controller_url.unwrap();
        let token = token.ok_or_else(|| anyhow::anyhow!("使用 --controller-url 时必须指定 --token"))?;

        info!("Controller mode: {}", controller_url);

        // Controller 模式：每次重连都重新获取配置
        loop {
            // 从 Controller 获取配置
            let cfg = match config::Config::from_controller(&controller_url, &token).await {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to fetch config from controller: {}", e);
                    warn!("Retrying in 5 seconds...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            info!("Config received: server={}:{}, protocol={:?}", cfg.server_addr, cfg.server_port, cfg.protocol);

            let server_addr = match cfg.get_server_addr() {
                Ok(addr) => addr,
                Err(e) => {
                    error!("Invalid server address: {}", e);
                    warn!("Retrying in 5 seconds...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            // 根据协议创建连接器
            let connector: Arc<dyn TunnelConnector> = match cfg.protocol {
                TunnelProtocol::Quic => {
                    info!("Using QUIC protocol");
                    match QuicConnector::new() {
                        Ok(c) => Arc::new(c),
                        Err(e) => {
                            error!("Failed to create QUIC connector: {}", e);
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    }
                }
                TunnelProtocol::Kcp => {
                    info!("Using KCP protocol");
                    Arc::new(KcpConnector::new(cfg.kcp.clone()))
                }
            };

            // 单次连接尝试
            match connector::connect_once(connector, server_addr, &cfg.token, log_collector.clone()).await {
                Ok(_) => info!("Connection closed"),
                Err(e) => error!("Connection error: {}", e),
            }

            warn!("Connection lost, reconnecting in 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    } else {
        // 配置文件模式
        let config_path = config_path.unwrap_or_else(|| "rfrpc.toml".to_string());
        let cfg = config::Config::from_file(&config_path)?;

        info!("Loaded configuration: {}", config_path);
        info!("Server address: {}:{}", cfg.server_addr, cfg.server_port);
        info!("Protocol: {:?}", cfg.protocol);

        let server_addr = cfg.get_server_addr()?;

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

        // 配置文件模式使用原有的重连循环
        connector::run(connector, server_addr, cfg.token, log_collector).await?;

        Ok(())
    }
}
