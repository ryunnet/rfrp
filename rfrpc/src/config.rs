use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub server_addr: String,
    pub server_port: u16,
    pub token: String,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件: {}", path_ref.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| "解析配置文件失败")?;

        Ok(config)
    }

    pub fn get_server_addr(&self) -> Result<SocketAddr> {
        let addr = format!("{}:{}", self.server_addr, self.server_port);
        addr.parse::<SocketAddr>()
            .with_context(|| format!("无效的服务器地址: {}", addr))
    }

    pub fn load_default() -> Result<Self> {
        Self::from_file("rfrpc.toml")
    }
}
