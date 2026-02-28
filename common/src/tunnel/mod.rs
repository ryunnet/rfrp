//! 隧道模块
//!
//! 此模块提供了统一的隧道抽象层，支持 QUIC、KCP 和 TCP 三种传输协议。
//! 通过 trait 抽象，客户端和服务端可以使用相同的接口处理不同协议的连接。

mod traits;
mod protocol;
mod quic;
mod kcp;
mod tcp;

pub use traits::*;
pub use protocol::*;
pub use quic::*;
pub use kcp::*;
pub use tcp::*;
