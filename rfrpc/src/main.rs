mod client;
mod config;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 tracing 日志系统
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    rustls::crypto::ring::default_provider().install_default().unwrap();

    // 读取配置文件
    let cfg = config::Config::load_default()?;

    info!("📋 加载配置文件: rfrpc.toml");
    info!("🌐 服务器地址: {}:{}", cfg.server_addr, cfg.server_port);
    info!("🔑 Token: {}", cfg.token);

    let server_addr = cfg.get_server_addr()?;

    // 运行客户端
    client::run(server_addr, cfg.token).await?;

    Ok(())
}
