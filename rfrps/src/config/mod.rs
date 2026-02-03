//! 服务端配置模块

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tokio::sync::OnceCell;

// 从共享库导入 KcpConfig
pub use rfrp_common::KcpConfig;

/// 服务端配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// QUIC 绑定端口
    pub bind_port: u16,
}

impl Config {
    /// 获取绑定地址字符串
    pub fn get_bind_addr(&self) -> String {
        format!("0.0.0.0:{}", self.bind_port)
    }
}

static CONFIG: OnceCell<Config> = OnceCell::const_new();

/// 获取全局配置
pub async fn get_config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}

/// 初始化配置
pub async fn init_config() -> Config {
    let path = Path::new("rfrps.toml");
    let content = fs::read_to_string(path)
        .with_context(|| format!("无法读取配置文件: {}", path.display()))
        .unwrap();

    let config: Config = toml::from_str(&content)
        .with_context(|| "解析配置文件失败")
        .unwrap();
    config
}
