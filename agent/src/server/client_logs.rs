use anyhow::Result;
use quinn::Connection;
use std::sync::Arc;
use tracing::{info};
use serde::{Deserialize, Serialize};

/// 日志条目（与客户端保持一致）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
}

/// 从客户端获取日志
pub async fn fetch_client_logs(
    conn: Arc<Connection>,
    count: u16,
) -> Result<Vec<LogEntry>> {
    // 打开双向QUIC流
    let (mut send, mut recv) = conn.open_bi().await?;

    // 发送日志请求消息
    // 格式: 1字节消息类型 + 2字节日志数量
    send.write_all(&[b'l']).await?; // 'l' = log request
    send.write_all(&count.to_be_bytes()).await?;
    send.finish()?;

    info!("📋 已发送日志请求，数量: {}", count);

    // 读取日志数据长度（4字节）
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    info!("📥 准备接收日志数据: {} 字节", len);

    // 读取日志数据
    let mut logs_buf = vec![0u8; len];
    recv.read_exact(&mut logs_buf).await?;

    // 反序列化日志
    let logs: Vec<LogEntry> = serde_json::from_slice(&logs_buf)?;

    info!("✅ 成功接收 {} 条日志", logs.len());

    Ok(logs)
}
