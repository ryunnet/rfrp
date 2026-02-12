//! 多节点管理器
//!
//! 管理多个 agent server 节点的连接，实现 ProxyControl trait，
//! 根据客户端所属节点自动路由操作到正确的节点。

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use common::protocol::control::{
    ConnectedClient, LogEntry, ProxyControl, ServerStatus,
};

use crate::entity::{Node, node, Client, client};
use crate::frps_client::RemoteProxyControl;
use crate::migration::get_connection;

/// 多节点管理器
///
/// 维护多个 RemoteProxyControl 实例，根据客户端所属节点路由操作。
pub struct NodeManager {
    /// node_id -> RemoteProxyControl 实例
    nodes: RwLock<HashMap<i64, Arc<RemoteProxyControl>>>,
}

impl NodeManager {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
        }
    }

    /// 从数据库加载所有节点，创建 RemoteProxyControl 实例
    pub async fn load_nodes(&self) -> Result<()> {
        let db = get_connection().await;
        let all_nodes = Node::find().all(db).await?;

        let mut nodes = self.nodes.write().await;
        nodes.clear();

        for node in all_nodes {
            let control = Arc::new(RemoteProxyControl::new(
                node.url.clone(),
                node.secret.clone(),
            ));
            info!("加载节点: #{} {} ({})", node.id, node.name, node.url);
            nodes.insert(node.id, control);
        }

        info!("共加载 {} 个节点", nodes.len());
        Ok(())
    }

    /// 动态添加或更新节点
    pub async fn add_node(&self, node_id: i64, url: String, secret: String) {
        let control = Arc::new(RemoteProxyControl::new(url, secret));
        let mut nodes = self.nodes.write().await;
        nodes.insert(node_id, control);
    }

    /// 动态移除节点
    pub async fn remove_node(&self, node_id: i64) {
        let mut nodes = self.nodes.write().await;
        nodes.remove(&node_id);
    }

    /// 获取指定节点的 ProxyControl
    pub async fn get_node_control(&self, node_id: i64) -> Option<Arc<RemoteProxyControl>> {
        let nodes = self.nodes.read().await;
        nodes.get(&node_id).cloned()
    }

    /// 根据 client_id 查找所属节点 ID
    async fn resolve_node_for_client(&self, client_id: &str) -> Result<Option<i64>> {
        let db = get_connection().await;
        let client_id_num: i64 = client_id.parse().unwrap_or(0);

        let client_model = Client::find_by_id(client_id_num)
            .one(db)
            .await?;

        Ok(client_model.and_then(|c| c.node_id))
    }

    /// 健康检查所有节点，返回 (node_id, is_online) 列表
    pub async fn check_all_nodes(&self) -> Vec<(i64, bool)> {
        let nodes = self.nodes.read().await;
        let mut results = Vec::new();

        for (&node_id, control) in nodes.iter() {
            let is_online = match control.get_server_status().await {
                Ok(_) => true,
                Err(e) => {
                    debug!("节点 #{} 健康检查失败: {}", node_id, e);
                    false
                }
            };
            results.push((node_id, is_online));
        }

        results
    }

    /// 获取所有已加载的节点 ID 列表
    pub async fn get_loaded_node_ids(&self) -> Vec<i64> {
        let nodes = self.nodes.read().await;
        nodes.keys().cloned().collect()
    }
}

#[async_trait]
impl ProxyControl for NodeManager {
    async fn start_proxy(&self, client_id: &str, proxy_id: i64) -> Result<()> {
        let node_id = self.resolve_node_for_client(client_id).await?
            .ok_or_else(|| anyhow!("客户端 {} 未关联任何节点", client_id))?;

        let control = self.get_node_control(node_id).await
            .ok_or_else(|| anyhow!("节点 #{} 未加载或不可用", node_id))?;

        control.start_proxy(client_id, proxy_id).await
    }

    async fn stop_proxy(&self, client_id: &str, proxy_id: i64) -> Result<()> {
        let node_id = self.resolve_node_for_client(client_id).await?
            .ok_or_else(|| anyhow!("客户端 {} 未关联任何节点", client_id))?;

        let control = self.get_node_control(node_id).await
            .ok_or_else(|| anyhow!("节点 #{} 未加载或不可用", node_id))?;

        control.stop_proxy(client_id, proxy_id).await
    }

    async fn get_connected_clients(&self) -> Result<Vec<ConnectedClient>> {
        let nodes = self.nodes.read().await;
        let mut all_clients = Vec::new();

        for (&node_id, control) in nodes.iter() {
            match control.get_connected_clients().await {
                Ok(clients) => {
                    all_clients.extend(clients);
                }
                Err(e) => {
                    warn!("从节点 #{} 获取连接客户端失败: {}", node_id, e);
                }
            }
        }

        Ok(all_clients)
    }

    async fn fetch_client_logs(&self, client_id: &str, count: u16) -> Result<Vec<LogEntry>> {
        let node_id = self.resolve_node_for_client(client_id).await?
            .ok_or_else(|| anyhow!("客户端 {} 未关联任何节点", client_id))?;

        let control = self.get_node_control(node_id).await
            .ok_or_else(|| anyhow!("节点 #{} 未加载或不可用", node_id))?;

        control.fetch_client_logs(client_id, count).await
    }

    async fn get_server_status(&self) -> Result<ServerStatus> {
        let nodes = self.nodes.read().await;
        let mut all_clients = Vec::new();
        let mut total_proxy_count = 0;

        for (&node_id, control) in nodes.iter() {
            match control.get_server_status().await {
                Ok(status) => {
                    all_clients.extend(status.connected_clients);
                    total_proxy_count += status.active_proxy_count;
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
