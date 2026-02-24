//! 多节点管理器
//!
//! 管理多个 agent server 节点的 gRPC 流连接，实现 ProxyControl trait，
//! 根据客户端所属节点自动路由操作到正确的节点。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};

use common::grpc::rfrp;
use common::grpc::rfrp::controller_to_agent_message::Payload as ControllerPayload;
use common::grpc::rfrp::agent_server_response::Result as AgentResult;
use common::grpc::pending_requests::PendingRequests;
use common::protocol::control::{
    ConnectedClient, LogEntry, ProxyControl, ServerStatus,
};

use crate::entity::{Client, Node};
use crate::migration::get_connection;

/// 单个节点的 gRPC 流连接
struct NodeStream {
    tx: mpsc::Sender<Result<rfrp::ControllerToAgentMessage, tonic::Status>>,
    pending: PendingRequests<rfrp::AgentServerResponse>,
}

/// 多节点管理器
pub struct NodeManager {
    /// node_id -> gRPC 流连接
    streams: RwLock<HashMap<i64, NodeStream>>,
}

impl NodeManager {
    pub fn new() -> Self {
        Self {
            streams: RwLock::new(HashMap::new()),
        }
    }

    /// 从数据库加载节点（gRPC 模式下仅用于初始化，实际连接由 Agent Server 主动发起）
    pub async fn load_nodes(&self) -> Result<()> {
        let db = get_connection().await;
        let all_nodes = Node::find().all(db).await?;
        info!("数据库中有 {} 个节点（等待 gRPC 连接）", all_nodes.len());
        Ok(())
    }

    /// 注册一个 Agent Server 的 gRPC 流
    pub async fn register_node_stream(
        &self,
        node_id: i64,
        tx: mpsc::Sender<Result<rfrp::ControllerToAgentMessage, tonic::Status>>,
    ) {
        let stream = NodeStream {
            tx,
            pending: PendingRequests::new(),
        };
        self.streams.write().await.insert(node_id, stream);
        info!("节点 #{} gRPC 流已注册", node_id);
    }

    /// 移除一个 Agent Server 的 gRPC 流
    pub async fn unregister_node_stream(&self, node_id: i64) {
        self.streams.write().await.remove(&node_id);
        info!("节点 #{} gRPC 流已移除", node_id);
    }

    /// 完成一个待处理的请求（由 AgentServerResponse 触发）
    pub async fn complete_pending_request(&self, node_id: i64, response: &rfrp::AgentServerResponse) {
        let streams = self.streams.read().await;
        if let Some(stream) = streams.get(&node_id) {
            stream.pending.complete(&response.request_id, response.clone()).await;
        }
    }

    /// 向指定节点发送命令并等待响应
    async fn send_command_and_wait(
        &self,
        node_id: i64,
        payload: ControllerPayload,
    ) -> Result<rfrp::AgentServerResponse> {
        let (request_id, rx, tx_clone) = {
            let streams = self.streams.read().await;
            let stream = streams.get(&node_id)
                .ok_or_else(|| anyhow!("节点 #{} 未连接", node_id))?;

            let (request_id, rx) = stream.pending.register().await;
            (request_id, rx, stream.tx.clone())
        };

        // 替换 payload 中的 request_id
        let final_payload = replace_request_id(payload, &request_id);

        let msg = rfrp::ControllerToAgentMessage {
            payload: Some(final_payload),
        };

        tx_clone.send(Ok(msg)).await
            .map_err(|_| anyhow!("发送命令到节点 #{} 失败", node_id))?;

        PendingRequests::wait(rx, Duration::from_secs(10)).await
    }

    /// 根据 client_id 查找所属节点 ID
    async fn resolve_node_for_client(&self, client_id: &str) -> Result<Option<i64>> {
        let db = get_connection().await;

        // 查找客户端的第一个启用的代理，并获取其节点ID
        let proxy = crate::entity::Proxy::find()
            .filter(crate::entity::proxy::Column::ClientId.eq(client_id))
            .filter(crate::entity::proxy::Column::Enabled.eq(true))
            .one(db)
            .await?;

        Ok(proxy.and_then(|p| p.node_id))
    }

    /// 健康检查所有节点
    pub async fn check_all_nodes(&self) -> Vec<(i64, bool)> {
        let db = get_connection().await;
        let all_nodes = match Node::find().all(db).await {
            Ok(nodes) => nodes,
            Err(e) => {
                warn!("查询节点列表失败: {}", e);
                return vec![];
            }
        };

        let streams = self.streams.read().await;

        all_nodes
            .into_iter()
            .map(|node| {
                let is_online = streams.contains_key(&node.id);
                (node.id, is_online)
            })
            .collect()
    }

    /// 获取所有已连接的节点 ID
    pub async fn get_loaded_node_ids(&self) -> Vec<i64> {
        let streams = self.streams.read().await;
        streams.keys().cloned().collect()
    }
}

/// 替换 payload 中的 request_id
fn replace_request_id(payload: ControllerPayload, request_id: &str) -> ControllerPayload {
    match payload {
        ControllerPayload::StartProxy(mut cmd) => {
            cmd.request_id = request_id.to_string();
            ControllerPayload::StartProxy(cmd)
        }
        ControllerPayload::StopProxy(mut cmd) => {
            cmd.request_id = request_id.to_string();
            ControllerPayload::StopProxy(cmd)
        }
        ControllerPayload::GetStatus(mut cmd) => {
            cmd.request_id = request_id.to_string();
            ControllerPayload::GetStatus(cmd)
        }
        ControllerPayload::GetClientLogs(mut cmd) => {
            cmd.request_id = request_id.to_string();
            ControllerPayload::GetClientLogs(cmd)
        }
        other => other,
    }
}

#[async_trait]
impl ProxyControl for NodeManager {
    async fn start_proxy(&self, client_id: &str, proxy_id: i64) -> Result<()> {
        let node_id = self.resolve_node_for_client(client_id).await?
            .ok_or_else(|| anyhow!("客户端 {} 未关联任何节点", client_id))?;

        let cmd = ControllerPayload::StartProxy(rfrp::StartProxyCommand {
            request_id: String::new(),
            client_id: client_id.to_string(),
            proxy_id,
        });

        let resp = self.send_command_and_wait(node_id, cmd).await?;

        match resp.result {
            Some(AgentResult::CommandAck(ack)) => {
                if ack.success {
                    Ok(())
                } else {
                    Err(anyhow!("启动代理失败: {}", ack.error.unwrap_or_default()))
                }
            }
            _ => Err(anyhow!("收到意外的响应类型")),
        }
    }

    async fn stop_proxy(&self, client_id: &str, proxy_id: i64) -> Result<()> {
        let node_id = self.resolve_node_for_client(client_id).await?
            .ok_or_else(|| anyhow!("客户端 {} 未关联任何节点", client_id))?;

        let cmd = ControllerPayload::StopProxy(rfrp::StopProxyCommand {
            request_id: String::new(),
            client_id: client_id.to_string(),
            proxy_id,
        });

        let resp = self.send_command_and_wait(node_id, cmd).await?;

        match resp.result {
            Some(AgentResult::CommandAck(ack)) => {
                if ack.success {
                    Ok(())
                } else {
                    Err(anyhow!("停止代理失败: {}", ack.error.unwrap_or_default()))
                }
            }
            _ => Err(anyhow!("收到意外的响应类型")),
        }
    }

    async fn get_connected_clients(&self) -> Result<Vec<ConnectedClient>> {
        let node_ids = self.get_loaded_node_ids().await;
        let mut all_clients = Vec::new();

        for node_id in node_ids {
            let cmd = ControllerPayload::GetStatus(rfrp::GetStatusCommand {
                request_id: String::new(),
            });

            match self.send_command_and_wait(node_id, cmd).await {
                Ok(resp) => {
                    if let Some(AgentResult::ServerStatus(status)) = resp.result {
                        for c in status.connected_clients {
                            all_clients.push(ConnectedClient {
                                client_id: c.client_id,
                                remote_address: c.remote_address,
                                protocol: c.protocol,
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!("从节点 #{} 获取状态失败: {}", node_id, e);
                }
            }
        }

        Ok(all_clients)
    }

    async fn fetch_client_logs(&self, client_id: &str, count: u16) -> Result<Vec<LogEntry>> {
        let node_id = self.resolve_node_for_client(client_id).await?
            .ok_or_else(|| anyhow!("客户端 {} 未关联任何节点", client_id))?;

        let cmd = ControllerPayload::GetClientLogs(rfrp::GetClientLogsCommand {
            request_id: String::new(),
            client_id: client_id.to_string(),
            count: count as u32,
        });

        let resp = self.send_command_and_wait(node_id, cmd).await?;

        match resp.result {
            Some(AgentResult::ClientLogs(logs)) => {
                Ok(logs.logs.into_iter().map(|l| LogEntry {
                    timestamp: l.timestamp,
                    level: l.level,
                    message: l.message,
                }).collect())
            }
            _ => Err(anyhow!("收到意外的响应类型")),
        }
    }

    async fn get_server_status(&self) -> Result<ServerStatus> {
        let node_ids = self.get_loaded_node_ids().await;
        let mut all_clients = Vec::new();
        let mut total_proxy_count = 0;

        for node_id in node_ids {
            let cmd = ControllerPayload::GetStatus(rfrp::GetStatusCommand {
                request_id: String::new(),
            });

            match self.send_command_and_wait(node_id, cmd).await {
                Ok(resp) => {
                    if let Some(AgentResult::ServerStatus(status)) = resp.result {
                        total_proxy_count += status.active_proxy_count as usize;
                        for c in status.connected_clients {
                            all_clients.push(ConnectedClient {
                                client_id: c.client_id,
                                remote_address: c.remote_address,
                                protocol: c.protocol,
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!("从节点 #{} 获取状态失败: {}", node_id, e);
                }
            }
        }

        Ok(ServerStatus {
            connected_clients: all_clients,
            active_proxy_count: total_proxy_count,
        })
    }
}
