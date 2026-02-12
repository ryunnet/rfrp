//! KCP 配置定义
//!
//! 此模块包含 KCP 协议的配置参数，用于客户端和服务端。

use serde::{Deserialize, Serialize};

/// KCP 协议配置
///
/// 用于配置 KCP 隧道的各项参数，包括延迟模式、发送间隔、
/// 快速重传和拥塞控制等。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KcpConfig {
    /// 是否启用 nodelay 模式
    /// - true: 低延迟模式，适合实时性要求高的场景
    /// - false: 正常模式，更高的吞吐量
    #[serde(default = "default_true")]
    pub nodelay: bool,

    /// 内部刷新时间间隔（毫秒）
    /// 默认值: 10ms，推荐范围 10-100ms
    #[serde(default = "default_interval")]
    pub interval: u32,

    /// 快速重传触发次数
    /// 当收到指定次数的 ACK 跨越某个包时触发快速重传
    /// 默认值: 2
    #[serde(default = "default_resend")]
    pub resend: u32,

    /// 是否关闭拥塞控制
    /// - true: 关闭拥塞控制，适合低延迟场景
    /// - false: 启用拥塞控制，更稳定但延迟更高
    #[serde(default = "default_true")]
    pub nc: bool,
}

fn default_true() -> bool {
    true
}

fn default_interval() -> u32 {
    10
}

fn default_resend() -> u32 {
    2
}

impl Default for KcpConfig {
    fn default() -> Self {
        Self {
            nodelay: true,
            interval: 10,
            resend: 2,
            nc: true,
        }
    }
}
