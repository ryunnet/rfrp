//! 客户端配置模块

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

// 从共享库导入
pub use rfrp_common::{TunnelProtocol, KcpConfig};

/// 客户端配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// 服务器地址
    pub server_addr: String,
    /// 服务器端口
    pub server_port: u16,
    /// 认证令牌
    pub token: String,
    /// 隧道协议类型
    #[serde(default)]
    pub protocol: TunnelProtocol,
    /// KCP 配置（可选）
    #[serde(default)]
    pub kcp: Option<KcpConfig>,
}

impl Config {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件: {}", path_ref.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| "解析配置文件失败")?;

        Ok(config)
    }

    /// 获取服务器地址
    pub fn get_server_addr(&self) -> Result<SocketAddr> {
        let addr = format!("{}:{}", self.server_addr, self.server_port);
        addr.parse::<SocketAddr>()
            .with_context(|| format!("无效的服务器地址: {}", addr))
    }

    /// 加载默认配置文件
    pub fn load_default() -> Result<Self> {
        Self::from_file("rfrpc.toml")
    }
}
