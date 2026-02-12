//! 客户端配置模块

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

// 从共享库导入
pub use common::{TunnelProtocol, KcpConfig};
use common::protocol::client_config::{ClientConnectConfig, ClientConnectConfigRequest};

/// 客户端运行时配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 服务器地址
    pub server_addr: String,
    /// 服务器端口
    pub server_port: u16,
    /// 认证令牌
    pub token: String,
    /// 隧道协议类型
    pub protocol: TunnelProtocol,
    /// KCP 配置（可选）
    pub kcp: Option<KcpConfig>,
}

/// 配置文件格式（兼容旧模式）
#[derive(Debug, Deserialize, Serialize, Clone)]
struct FileConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub token: String,
    #[serde(default)]
    pub protocol: TunnelProtocol,
    #[serde(default)]
    pub kcp: Option<KcpConfig>,
}

impl Config {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件: {}", path_ref.display()))?;

        let file_config: FileConfig = toml::from_str(&content)
            .with_context(|| "解析配置文件失败")?;

        Ok(Config {
            server_addr: file_config.server_addr,
            server_port: file_config.server_port,
            token: file_config.token,
            protocol: file_config.protocol,
            kcp: file_config.kcp,
        })
    }

    /// 从 Controller 获取配置
    pub async fn from_controller(controller_url: &str, token: &str) -> Result<Self> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/client/connect-config", controller_url.trim_end_matches('/'));

        let req = ClientConnectConfigRequest {
            token: token.to_string(),
        };

        let resp = client.post(&url)
            .json(&req)
            .send()
            .await
            .with_context(|| format!("无法连接到 Controller: {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Controller 返回错误 ({}): {}", status, body));
        }

        let config: ClientConnectConfig = resp.json()
            .await
            .with_context(|| "解析 Controller 返回的配置失败")?;

        Ok(Config {
            server_addr: config.server_addr,
            server_port: config.server_port,
            token: token.to_string(),
            protocol: config.protocol,
            kcp: config.kcp,
        })
    }

    /// 获取服务器地址
    pub fn get_server_addr(&self) -> Result<SocketAddr> {
        let addr = format!("{}:{}", self.server_addr, self.server_port);
        addr.parse::<SocketAddr>()
            .with_context(|| format!("无效的服务器地址: {}", addr))
    }
}
