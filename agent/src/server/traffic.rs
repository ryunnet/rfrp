use std::collections::HashMap;
use tracing::{debug, error};
use tokio::sync::mpsc;
use std::time::Duration;

use common::grpc::rfrp;
use common::grpc::rfrp::agent_server_message::Payload as AgentPayload;

use super::grpc_client::SharedGrpcSender;

struct TrafficEvent {
    proxy_id: i64,
    client_id: i64,
    user_id: Option<i64>,
    bytes_sent: i64,
    bytes_received: i64,
}

/// 流量统计管理器（通过 gRPC 流上报到 Controller）
#[derive(Clone)]
pub struct TrafficManager {
    sender: mpsc::Sender<TrafficEvent>,
}

impl TrafficManager {
    /// 创建 gRPC 模式的 TrafficManager
    pub fn new(grpc_sender: SharedGrpcSender) -> Self {
        let (tx, mut rx) = mpsc::channel::<TrafficEvent>(10000);

        tokio::spawn(async move {
            let mut buffer: HashMap<(i64, i64, Option<i64>), (i64, i64)> = HashMap::new();
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        let key = (event.proxy_id, event.client_id, event.user_id);
                        let entry = buffer.entry(key).or_insert((0, 0));
                        entry.0 += event.bytes_sent;
                        entry.1 += event.bytes_received;

                        if buffer.len() > 100 {
                            Self::flush_buffer_grpc(&grpc_sender, &mut buffer).await;
                        }
                    }
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            Self::flush_buffer_grpc(&grpc_sender, &mut buffer).await;
                        }
                    }
                }
            }
        });

        Self { sender: tx }
    }

    /// 通过 gRPC 流发送流量上报
    async fn flush_buffer_grpc(
        grpc_sender: &SharedGrpcSender,
        buffer: &mut HashMap<(i64, i64, Option<i64>), (i64, i64)>,
    ) {
        let records: Vec<rfrp::TrafficRecord> = buffer
            .drain()
            .filter(|(_, (sent, recv))| *sent > 0 || *recv > 0)
            .map(|((proxy_id, client_id, user_id), (bytes_sent, bytes_received))| {
                rfrp::TrafficRecord {
                    proxy_id,
                    client_id: client_id.to_string(),
                    user_id,
                    bytes_sent,
                    bytes_received,
                }
            })
            .collect();

        if records.is_empty() {
            return;
        }

        let count = records.len();
        let msg = rfrp::AgentServerMessage {
            payload: Some(AgentPayload::TrafficReport(rfrp::TrafficReportRequest {
                records,
            })),
        };

        match grpc_sender.send(msg).await {
            Ok(()) => {
                debug!("gRPC 上报流量: {} 条记录", count);
            }
            Err(e) => {
                error!("gRPC 上报流量失败: {}", e);
            }
        }
    }

    /// 实时记录流量统计 (异步非阻塞)
    pub async fn record_traffic(
        &self,
        proxy_id: i64,
        client_id: i64,
        user_id: Option<i64>,
        bytes_sent: i64,
        bytes_received: i64,
    ) {
        if bytes_sent == 0 && bytes_received == 0 {
            return;
        }

        let event = TrafficEvent {
            proxy_id,
            client_id,
            user_id,
            bytes_sent,
            bytes_received,
        };

        if let Err(e) = self.sender.send(event).await {
            error!("发送流量统计事件失败: {}", e);
        }
    }
}
