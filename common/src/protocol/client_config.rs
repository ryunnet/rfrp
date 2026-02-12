//! 客户端连接配置协议类型
//!
//! 定义了 Agent Client 从 Controller 获取连接配置的请求/响应结构体。

use serde::{Deserialize, Serialize};
use crate::config::KcpConfig;
use crate::tunnel::TunnelProtocol;

/// 客户端连接配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConnectConfigRequest {
    pub token: String,
}

/// Controller 返回给 Agent Client 的连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConnectConfig {
    /// Agent Server 地址
    pub server_addr: String,
    /// Agent Server 端口
    pub server_port: u16,
    /// 隧道协议类型
    pub protocol: TunnelProtocol,
    /// KCP 配置（可选）
    pub kcp: Option<KcpConfig>,
    /// 客户端 ID
    pub client_id: i64,
    /// 客户端名称
    pub client_name: String,
}
