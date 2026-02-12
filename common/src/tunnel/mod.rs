//! 隧道模块
//!
//! 此模块提供了统一的隧道抽象层，支持 QUIC 和 KCP 两种传输协议。
//! 通过 trait 抽象，客户端和服务端可以使用相同的接口处理不同协议的连接。

mod traits;
mod protocol;
mod quic;
mod kcp;

pub use traits::*;
pub use protocol::*;
pub use quic::*;
pub use kcp::*;
