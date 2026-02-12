//! 流量上报相关类型
//!
//! 定义了 frps 向 Controller 上报流量数据的结构体。

use serde::{Deserialize, Serialize};

/// 单条流量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficRecord {
    pub proxy_id: i64,
    pub client_id: String,
    pub user_id: Option<i64>,
    pub bytes_sent: i64,
    pub bytes_received: i64,
}

/// 批量流量上报请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficReportRequest {
    pub records: Vec<TrafficRecord>,
}

/// 流量上报响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficReportResponse {
    pub accepted: bool,
}
