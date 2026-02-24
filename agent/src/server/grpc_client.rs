//! Agent Server gRPC Client
//!
//! 连接 Controller 的 gRPC 双向流，处理认证、消息分发、命令执行。

use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, Result};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tracing::{error, info, warn};

use common::grpc::rfrp;
use common::grpc::rfrp::agent_server_message::Payload as AgentPayload;
use common::grpc::rfrp::controller_to_agent_message::Payload as ControllerPayload;
use common::grpc::rfrp::agent_server_response::Result as AgentResult;
use common::grpc::AgentServerServiceClient;
use common::grpc::pending_requests::PendingRequests;
use common::protocol::control::ProxyControl;

/// gRPC 流发送器类型
pub type GrpcSender = mpsc::Sender<rfrp::AgentServerMessage>;

/// 可热替换的 gRPC 发送器（重连后更新内部 sender）
#[derive(Clone)]
pub struct SharedGrpcSender {
    inner: Arc<RwLock<GrpcSender>>,
}

impl SharedGrpcSender {
    pub fn new(sender: GrpcSender) -> Self {
        Self {
            inner: Arc::new(RwLock::new(sender)),
        }
    }

    /// 发送消息（使用当前 sender）
    pub async fn send(&self, msg: rfrp::AgentServerMessage) -> Result<(), mpsc::error::SendError<rfrp::AgentServerMessage>> {
        let sender = self.inner.read().await;
        sender.send(msg).await
    }

    /// 重连后替换内部 sender
    pub async fn replace(&self, new_sender: GrpcSender) {
        let mut sender = self.inner.write().await;
        *sender = new_sender;
    }
}

/// 可热替换的 PendingRequests（重连后更新）
#[derive(Clone)]
pub struct SharedPendingRequests {
    inner: Arc<RwLock<PendingRequests<ControllerResponse>>>,
}

impl SharedPendingRequests {
    pub fn new(pending: PendingRequests<ControllerResponse>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(pending)),
        }
    }

    /// 注册一个等待响应的请求
    pub async fn register(&self) -> (String, tokio::sync::oneshot::Receiver<ControllerResponse>) {
        let pending = self.inner.read().await;
        pending.register().await
    }

    /// 重连后替换内部 pending
    pub async fn replace(&self, new_pending: PendingRequests<ControllerResponse>) {
        let mut pending = self.inner.write().await;
        *pending = new_pending;
    }

    /// 获取内部 PendingRequests 的克隆（供 message_loop 使用）
    pub async fn get_inner(&self) -> PendingRequests<ControllerResponse> {
        self.inner.read().await.clone()
    }
}

/// Agent Server gRPC 客户端
pub struct AgentGrpcClient {
    /// 可热替换的发送器
    shared_sender: SharedGrpcSender,
    /// 可热替换的 pending requests
    shared_pending: SharedPendingRequests,
    /// 节点 ID（连接认证后获得）
    node_id: RwLock<i64>,
}

/// Controller 响应的包装类型
#[derive(Clone)]
pub enum ControllerResponse {
    ValidateToken(rfrp::ValidateTokenResponse),
    ClientOnline(rfrp::ClientOnlineResponse),
    TrafficLimit(rfrp::TrafficLimitResponse),
    GetClientProxies(rfrp::GetClientProxiesResponse),
    TrafficReport(rfrp::TrafficReportResponse),
}

impl AgentGrpcClient {
    /// 连接 Controller 并认证节点
    pub async fn connect_and_authenticate(
        controller_url: &str,
        token: &str,
        tunnel_port: u16,
        tunnel_protocol: &str,
    ) -> Result<(Arc<Self>, mpsc::Receiver<ControllerCommand>)> {
        let channel = Channel::from_shared(controller_url.to_string())?
            .connect()
            .await
            .map_err(|e| anyhow!("连接 Controller gRPC 失败: {}", e))?;

        let mut client = AgentServerServiceClient::new(channel);

        // 创建双向流
        let (tx, rx) = mpsc::channel::<rfrp::AgentServerMessage>(256);
        let (cmd_tx, cmd_rx) = mpsc::channel::<ControllerCommand>(64);

        let pending = PendingRequests::<ControllerResponse>::new();

        // 发送认证请求作为首条消息
        let register_msg = rfrp::AgentServerMessage {
            payload: Some(AgentPayload::Register(rfrp::NodeRegisterRequest {
                token: token.to_string(),
                tunnel_port: tunnel_port as u32,
                tunnel_protocol: tunnel_protocol.to_string(),
            })),
        };
        tx.send(register_msg).await
            .map_err(|_| anyhow!("发送认证消息失败"))?;

        // 建立 gRPC 流
        let outbound = tokio_stream::wrappers::ReceiverStream::new(rx);
        let response = client.agent_server_channel(outbound).await
            .map_err(|e| anyhow!("建立 gRPC 流失败: {}", e))?;

        let mut inbound = response.into_inner();

        // 读取认证响应
        let first_msg = inbound.next().await
            .ok_or_else(|| anyhow!("未收到认证响应"))?
            .map_err(|e| anyhow!("读取认证响应失败: {}", e))?;

        let register_resp = match first_msg.payload {
            Some(ControllerPayload::RegisterResponse(resp)) => resp,
            _ => return Err(anyhow!("首条响应不是认证响应")),
        };

        let node_id = register_resp.node_id;
        info!("gRPC 连接认证成功: 节点 #{} ({})", node_id, register_resp.node_name);

        let shared_sender = SharedGrpcSender::new(tx.clone());
        let shared_pending = SharedPendingRequests::new(pending.clone());

        let grpc_client = Arc::new(Self {
            shared_sender,
            shared_pending,
            node_id: RwLock::new(node_id),
        });

        // 启动消息接收循环
        let pending_clone = pending.clone();
        let cmd_tx_clone = cmd_tx.clone();

        tokio::spawn(async move {
            Self::message_loop(inbound, pending_clone, cmd_tx_clone, tx, node_id).await;
        });

        // 启动心跳
        let heartbeat_sender = grpc_client.shared_sender.clone();
        tokio::spawn(async move {
            Self::shared_heartbeat_loop(heartbeat_sender).await;
        });

        Ok((grpc_client, cmd_rx))
    }

    /// 重连 Controller（复用已有的 SharedGrpcSender 和 SharedPendingRequests）
    pub async fn reconnect(
        self: &Arc<Self>,
        controller_url: &str,
        token: &str,
        tunnel_port: u16,
        tunnel_protocol: &str,
    ) -> Result<mpsc::Receiver<ControllerCommand>> {
        let channel = Channel::from_shared(controller_url.to_string())?
            .connect()
            .await
            .map_err(|e| anyhow!("重连 Controller gRPC 失败: {}", e))?;

        let mut client = AgentServerServiceClient::new(channel);

        // 创建新的双向流
        let (tx, rx) = mpsc::channel::<rfrp::AgentServerMessage>(256);
        let (cmd_tx, cmd_rx) = mpsc::channel::<ControllerCommand>(64);

        let pending = PendingRequests::<ControllerResponse>::new();

        // 发送认证请求
        let register_msg = rfrp::AgentServerMessage {
            payload: Some(AgentPayload::Register(rfrp::NodeRegisterRequest {
                token: token.to_string(),
                tunnel_port: tunnel_port as u32,
                tunnel_protocol: tunnel_protocol.to_string(),
            })),
        };
        tx.send(register_msg).await
            .map_err(|_| anyhow!("发送认证消息失败"))?;

        // 建立 gRPC 流
        let outbound = tokio_stream::wrappers::ReceiverStream::new(rx);
        let response = client.agent_server_channel(outbound).await
            .map_err(|e| anyhow!("建立 gRPC 流失败: {}", e))?;

        let mut inbound = response.into_inner();

        // 读取认证响应
        let first_msg = inbound.next().await
            .ok_or_else(|| anyhow!("未收到认证响应"))?
            .map_err(|e| anyhow!("读取认证响应失败: {}", e))?;

        let register_resp = match first_msg.payload {
            Some(ControllerPayload::RegisterResponse(resp)) => resp,
            _ => return Err(anyhow!("首条响应不是认证响应")),
        };

        let node_id = register_resp.node_id;
        info!("gRPC 重连认证成功: 节点 #{} ({})", node_id, register_resp.node_name);

        // 热替换 sender 和 pending
        self.shared_sender.replace(tx.clone()).await;
        self.shared_pending.replace(pending.clone()).await;
        *self.node_id.write().await = node_id;

        // 启动新的消息接收循环
        let pending_clone = pending.clone();
        let cmd_tx_clone = cmd_tx.clone();

        tokio::spawn(async move {
            Self::message_loop(inbound, pending_clone, cmd_tx_clone, tx, node_id).await;
        });

        // 启动新的心跳（旧的会因为 sender 被替换而自动停止）
        let heartbeat_sender = self.shared_sender.clone();
        tokio::spawn(async move {
            Self::shared_heartbeat_loop(heartbeat_sender).await;
        });

        Ok(cmd_rx)
    }

    /// 消息接收循环
    async fn message_loop(
        mut inbound: tonic::Streaming<rfrp::ControllerToAgentMessage>,
        pending: PendingRequests<ControllerResponse>,
        cmd_tx: mpsc::Sender<ControllerCommand>,
        _tx: GrpcSender,
        node_id: i64,
    ) {
        while let Some(result) = inbound.next().await {
            let msg = match result {
                Ok(m) => m,
                Err(e) => {
                    error!("gRPC 流错误: {}", e);
                    break;
                }
            };

            let payload = match msg.payload {
                Some(p) => p,
                None => continue,
            };

            match payload {
                ControllerPayload::HeartbeatResponse(_) => {
                    // 心跳响应，忽略
                }

                ControllerPayload::ValidateTokenResponse(resp) => {
                    let rid = resp.request_id.clone();
                    pending.complete(&rid, ControllerResponse::ValidateToken(resp)).await;
                }

                ControllerPayload::ClientOnlineResponse(resp) => {
                    let rid = resp.request_id.clone();
                    pending.complete(&rid, ControllerResponse::ClientOnline(resp)).await;
                }

                ControllerPayload::TrafficLimitResponse(resp) => {
                    let rid = resp.request_id.clone();
                    pending.complete(&rid, ControllerResponse::TrafficLimit(resp)).await;
                }

                ControllerPayload::GetClientProxiesResponse(resp) => {
                    let rid = resp.request_id.clone();
                    pending.complete(&rid, ControllerResponse::GetClientProxies(resp)).await;
                }

                ControllerPayload::TrafficReportResponse(_resp) => {
                    // 流量上报是 fire-and-forget，无需关联响应
                }

                // Controller 主动下发的指令
                ControllerPayload::StartProxy(cmd) => {
                    let _ = cmd_tx.send(ControllerCommand::StartProxy {
                        request_id: cmd.request_id,
                        client_id: cmd.client_id,
                        proxy_id: cmd.proxy_id,
                    }).await;
                }

                ControllerPayload::StopProxy(cmd) => {
                    let _ = cmd_tx.send(ControllerCommand::StopProxy {
                        request_id: cmd.request_id,
                        client_id: cmd.client_id,
                        proxy_id: cmd.proxy_id,
                    }).await;
                }

                ControllerPayload::GetStatus(cmd) => {
                    let _ = cmd_tx.send(ControllerCommand::GetStatus {
                        request_id: cmd.request_id,
                    }).await;
                }

                ControllerPayload::GetClientLogs(cmd) => {
                    let _ = cmd_tx.send(ControllerCommand::GetClientLogs {
                        request_id: cmd.request_id,
                        client_id: cmd.client_id,
                        count: cmd.count,
                    }).await;
                }

                _ => {
                    warn!("收到未知的 Controller 消息类型");
                }
            }
        }

        warn!("节点 #{} gRPC 连接断开", node_id);
    }

    /// 心跳循环（使用 SharedGrpcSender）
    async fn shared_heartbeat_loop(sender: SharedGrpcSender) {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        interval.tick().await; // 跳过首次

        loop {
            interval.tick().await;

            let msg = rfrp::AgentServerMessage {
                payload: Some(AgentPayload::Heartbeat(rfrp::Heartbeat {
                    timestamp: chrono::Utc::now().timestamp(),
                })),
            };

            if sender.send(msg).await.is_err() {
                warn!("心跳发送失败，连接可能已断开");
                break;
            }
        }
    }

    /// 获取节点 ID
    pub async fn node_id(&self) -> i64 {
        *self.node_id.read().await
    }

    /// 获取共享发送器（供 auth provider 和 traffic manager 使用）
    pub fn shared_sender(&self) -> &SharedGrpcSender {
        &self.shared_sender
    }

    /// 获取共享 pending requests（供 auth provider 使用）
    pub fn shared_pending(&self) -> &SharedPendingRequests {
        &self.shared_pending
    }

    /// 发送命令响应到 Controller
    pub async fn send_response(&self, response: rfrp::AgentServerResponse) -> Result<()> {
        let msg = rfrp::AgentServerMessage {
            payload: Some(AgentPayload::Response(response)),
        };
        self.shared_sender.send(msg).await
            .map_err(|_| anyhow!("发送响应失败"))?;
        Ok(())
    }
}

/// Controller 下发的命令
pub enum ControllerCommand {
    StartProxy {
        request_id: String,
        client_id: String,
        proxy_id: i64,
    },
    StopProxy {
        request_id: String,
        client_id: String,
        proxy_id: i64,
    },
    GetStatus {
        request_id: String,
    },
    GetClientLogs {
        request_id: String,
        client_id: String,
        count: u32,
    },
}

/// 命令处理器：处理 Controller 下发的命令并发送响应
pub async fn handle_controller_commands(
    mut cmd_rx: mpsc::Receiver<ControllerCommand>,
    grpc_client: Arc<AgentGrpcClient>,
    proxy_control: Arc<dyn ProxyControl>,
) {
    while let Some(cmd) = cmd_rx.recv().await {
        let grpc = grpc_client.clone();
        let control = proxy_control.clone();

        tokio::spawn(async move {
            match cmd {
                ControllerCommand::StartProxy { request_id, client_id, proxy_id } => {
                    let result = control.start_proxy(&client_id, proxy_id).await;
                    let ack = match result {
                        Ok(()) => rfrp::CommandAck { success: true, error: None },
                        Err(e) => rfrp::CommandAck { success: false, error: Some(e.to_string()) },
                    };
                    let resp = rfrp::AgentServerResponse {
                        request_id,
                        result: Some(AgentResult::CommandAck(ack)),
                    };
                    let _ = grpc.send_response(resp).await;
                }

                ControllerCommand::StopProxy { request_id, client_id, proxy_id } => {
                    let result = control.stop_proxy(&client_id, proxy_id).await;
                    let ack = match result {
                        Ok(()) => rfrp::CommandAck { success: true, error: None },
                        Err(e) => rfrp::CommandAck { success: false, error: Some(e.to_string()) },
                    };
                    let resp = rfrp::AgentServerResponse {
                        request_id,
                        result: Some(AgentResult::CommandAck(ack)),
                    };
                    let _ = grpc.send_response(resp).await;
                }

                ControllerCommand::GetStatus { request_id } => {
                    let result = control.get_server_status().await;
                    let resp = match result {
                        Ok(status) => {
                            let clients: Vec<rfrp::ConnectedClient> = status.connected_clients
                                .into_iter()
                                .map(|c| rfrp::ConnectedClient {
                                    client_id: c.client_id,
                                    remote_address: c.remote_address,
                                    protocol: c.protocol,
                                })
                                .collect();
                            rfrp::AgentServerResponse {
                                request_id,
                                result: Some(AgentResult::ServerStatus(rfrp::ServerStatus {
                                    connected_clients: clients,
                                    active_proxy_count: status.active_proxy_count as u32,
                                })),
                            }
                        }
                        Err(e) => rfrp::AgentServerResponse {
                            request_id,
                            result: Some(AgentResult::CommandAck(rfrp::CommandAck {
                                success: false,
                                error: Some(e.to_string()),
                            })),
                        },
                    };
                    let _ = grpc.send_response(resp).await;
                }

                ControllerCommand::GetClientLogs { request_id, client_id, count } => {
                    let result = control.fetch_client_logs(&client_id, count as u16).await;
                    let resp = match result {
                        Ok(logs) => {
                            let entries: Vec<rfrp::LogEntry> = logs
                                .into_iter()
                                .map(|l| rfrp::LogEntry {
                                    timestamp: l.timestamp,
                                    level: l.level,
                                    message: l.message,
                                })
                                .collect();
                            rfrp::AgentServerResponse {
                                request_id,
                                result: Some(AgentResult::ClientLogs(rfrp::ClientLogsResponse {
                                    logs: entries,
                                })),
                            }
                        }
                        Err(e) => rfrp::AgentServerResponse {
                            request_id,
                            result: Some(AgentResult::CommandAck(rfrp::CommandAck {
                                success: false,
                                error: Some(e.to_string()),
                            })),
                        },
                    };
                    let _ = grpc.send_response(resp).await;
                }
            }
        });
    }
}
