use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tokio::sync::OnceCell;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub bind_port: u16,
}

impl Config {
    pub fn get_bind_addr(&self) -> String {
        format!("0.0.0.0:{}", self.bind_port)
    }
}

static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub async fn get_config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}

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