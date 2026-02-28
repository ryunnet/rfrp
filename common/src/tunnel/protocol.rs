//! 隧道协议类型定义

use serde::{Deserialize, Serialize};

/// 隧道协议类型
///
/// 支持 QUIC、KCP 和 TCP 三种传输协议：
/// - QUIC: 基于 UDP 的多路复用安全传输协议，默认选项
/// - KCP: 快速可靠的 UDP 传输协议，适合高延迟网络
/// - TCP: 基于 TCP 的传输协议（yamux 多路复用），适合 UDP 受限网络
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TunnelProtocol {
    /// QUIC 协议（默认）
    #[default]
    Quic,
    /// KCP 协议
    Kcp,
    /// TCP 协议（yamux 多路复用）
    Tcp,
}

impl std::fmt::Display for TunnelProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelProtocol::Quic => write!(f, "quic"),
            TunnelProtocol::Kcp => write!(f, "kcp"),
            TunnelProtocol::Tcp => write!(f, "tcp"),
        }
    }
}
