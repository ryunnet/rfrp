//! RFRP 公共库
//!
//! 此库包含 rfrpc（客户端）和 rfrps（服务端）共享的代码，
//! 包括隧道协议抽象、KCP 配置和协议类型定义。

pub mod tunnel;
pub mod config;

pub use tunnel::{
    TunnelProtocol,
    TunnelSendStream,
    TunnelRecvStream,
    TunnelConnection,
    TunnelConnector,
    TunnelListener,
    QuicSendStream,
    QuicRecvStream,
    QuicConnection,
    QuicConnector,
    QuicListener,
    KcpSendStream,
    KcpRecvStream,
    KcpConnection,
    KcpConnector,
    KcpListener,
    KcpMultiplexer,
};

pub use config::KcpConfig;
