//! 隧道协议类型定义

use serde::{Deserialize, Serialize};

/// 隧道协议类型
///
/// 支持 QUIC 和 KCP 两种传输协议：
/// - QUIC: 基于 UDP 的多路复用安全传输协议，默认选项
/// - KCP: 快速可靠的 UDP 传输协议，适合高延迟网络
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TunnelProtocol {
    /// QUIC 协议（默认）
    #[default]
    Quic,
    /// KCP 协议
    Kcp,
}

impl std::fmt::Display for TunnelProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelProtocol::Quic => write!(f, "quic"),
            TunnelProtocol::Kcp => write!(f, "kcp"),
        }
    }
}
