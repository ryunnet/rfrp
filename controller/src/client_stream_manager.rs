//! Agent Client 流管理器
//!
//! 管理所有已连接的 Agent Client gRPC 流，
//! 当代理配置变更时推送 ProxyListUpdate 通知。

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use common::grpc::rfrp;
use common::KcpConfig;

use crate::entity::{Client, Node, Proxy, client, proxy, node};
use crate::migration::get_connection;

/// 管理已连接的 Agent Client 流
#[derive(Clone)]
pub struct ClientStreamManager {
    /// client_id -> stream sender
    streams: Arc<RwLock<HashMap<i64, mpsc::Sender<Result<rfrp::ControllerToClientMessage, tonic::Status>>>>>,
}

impl ClientStreamManager {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册一个 Agent Client 流
    pub async fn register(
        &self,
        client_id: i64,
        tx: mpsc::Sender<Result<rfrp::ControllerToClientMessage, tonic::Status>>,
    ) {
        info!("Agent Client #{} 已连接", client_id);
        self.streams.write().await.insert(client_id, tx);
    }

    /// 移除一个 Agent Client 流
    pub async fn unregister(&self, client_id: i64) {
        info!("Agent Client #{} 已断开", client_id);
        self.streams.write().await.remove(&client_id);
    }

    /// 通知指定客户端代理配置已变更
    pub async fn notify_proxy_change(&self, client_id_str: &str) {
        let client_id: i64 = match client_id_str.parse() {
            Ok(id) => id,
            Err(_) => return,
        };

        let update = match self.build_proxy_list_update(client_id).await {
            Ok(u) => u,
            Err(e) => {
                error!("构建代理列表更新失败: {}", e);
                return;
            }
        };

        let streams = self.streams.read().await;
        if let Some(tx) = streams.get(&client_id) {
            let msg = rfrp::ControllerToClientMessage {
                payload: Some(rfrp::controller_to_client_message::Payload::ProxyUpdate(update)),
            };
            if let Err(e) = tx.send(Ok(msg)).await {
                error!("推送代理更新到 Client #{} 失败: {}", client_id, e);
            } else {
                debug!("已推送代理更新到 Client #{}", client_id);
            }
        }
    }

    /// 健康检查所有客户端
    pub async fn check_all_clients(&self) -> Vec<(i64, bool)> {
        let db = get_connection().await;
        let all_clients = match Client::find().all(db).await {
            Ok(clients) => clients,
            Err(e) => {
                error!("查询客户端列表失败: {}", e);
                return vec![];
            }
        };

        let streams = self.streams.read().await;

        all_clients
            .into_iter()
            .map(|client| {
                let is_online = streams.contains_key(&client.id);
                (client.id, is_online)
            })
            .collect()
    }

    /// 构建代理列表更新消息
    pub async fn build_proxy_list_update(&self, client_id: i64) -> anyhow::Result<rfrp::ProxyListUpdate> {
        let db = get_connection().await;

        // 查询客户端
        let client_model = Client::find_by_id(client_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("客户端 #{} 不存在", client_id))?;

        // 查询所有已启用代理
        let proxies = Proxy::find()
            .filter(proxy::Column::ClientId.eq(client_id.to_string()))
            .filter(proxy::Column::Enabled.eq(true))
            .all(db)
            .await?;

        // 按 node_id 分组
        let mut node_proxy_map: HashMap<i64, Vec<rfrp::ProxyInfo>> = HashMap::new();
        for p in &proxies {
            let effective_node_id = p.node_id.or(client_model.node_id);
            let nid = match effective_node_id {
                Some(id) => id,
                None => continue,
            };

            node_proxy_map
                .entry(nid)
                .or_default()
                .push(rfrp::ProxyInfo {
                    proxy_id: p.id,
                    name: p.name.clone(),
                    proxy_type: p.proxy_type.clone(),
                    local_ip: p.local_ip.clone(),
                    local_port: p.local_port as i32,
                    remote_port: p.remote_port as i32,
                    enabled: p.enabled,
                });
        }

        // 查询节点信息
        let node_ids: Vec<i64> = node_proxy_map.keys().cloned().collect();
        let nodes = if node_ids.is_empty() {
            vec![]
        } else {
            Node::find()
                .filter(node::Column::Id.is_in(node_ids))
                .all(db)
                .await?
        };

        let mut server_groups = Vec::new();
        for n in nodes {
            if let Some(proxy_list) = node_proxy_map.remove(&n.id) {
                let kcp = n.kcp_config
                    .as_deref()
                    .and_then(|s| serde_json::from_str::<KcpConfig>(s).ok())
                    .map(|k| rfrp::GrpcKcpConfig {
                        nodelay: k.nodelay,
                        interval: k.interval,
                        resend: k.resend,
                        nc: k.nc,
                    });

                server_groups.push(rfrp::ServerProxyGroup {
                    node_id: n.id,
                    server_addr: n.tunnel_addr,
                    server_port: n.tunnel_port as u32,
                    protocol: n.tunnel_protocol,
                    kcp,
                    proxies: proxy_list,
                });
            }
        }

        Ok(rfrp::ProxyListUpdate {
            client_id: client_model.id,
            client_name: client_model.name,
            server_groups,
        })
    }
}
