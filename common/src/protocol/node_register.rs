//! 节点注册相关类型
//!
//! 定义了 Agent Server 向 Controller 注册的请求/响应结构体。

use serde::{Deserialize, Serialize};

/// Agent Server 注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegisterRequest {
    /// 节点密钥（对应 controller 中 node.secret）
    pub token: String,
    /// 隧道监听端口
    pub tunnel_port: u16,
    /// 内部 API 端口
    pub internal_port: u16,
    /// 隧道协议 ("quic" 或 "kcp")
    pub tunnel_protocol: String,
}

/// Controller 注册响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegisterResponse {
    /// 节点 ID
    pub node_id: i64,
    /// 节点名称
    pub node_name: String,
    /// 内部 API 密钥（agent server 用于验证 controller 的调用）
    pub internal_secret: String,
    /// Controller 内部 API 地址（agent server 用于回调 controller）
    pub controller_internal_url: String,
}
