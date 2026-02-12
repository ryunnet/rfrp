//! 内部通信协议类型定义
//!
//! 此模块定义了 Controller 和 frps 之间通信的共享类型，
//! 包括 ProxyControl trait、ClientAuthProvider trait 以及相关的请求/响应结构体。

pub mod control;
pub mod auth;
pub mod traffic;
pub mod client_config;
pub mod node_register;
