//! 连接管理器
//!
//! 管理到多个 Agent Server 的隧道连接。
//! 根据 Controller 返回的代理列表，动态建立和断开连接。

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{info, error, warn};

use common::{TunnelConnector, QuicConnector, KcpConnector, TunnelProtocol};
use common::protocol::client_config::ServerProxyGroup;

use crate::client::connector;
use crate::client::log_collector::LogCollector;

/// 单个 Server 连接的状态
struct ServerConnection {
    node_id: i64,
    proxy_ids: HashSet<i64>,
    cancel_token: tokio_util::sync::CancellationToken,
    handle: JoinHandle<()>,
}

/// 连接管理器
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<i64, ServerConnection>>>,
    token: String,
    log_collector: LogCollector,
}

impl ConnectionManager {
    pub fn new(token: String, log_collector: LogCollector) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            token,
            log_collector,
        }
    }

    /// 根据新的代理分组列表，调和（reconcile）连接状态
    pub async fn reconcile(&self, server_groups: Vec<ServerProxyGroup>) {
        let new_node_ids: HashSet<i64> = server_groups.iter().map(|g| g.node_id).collect();

        // 1. 断开不再需要的连接
        let nodes_to_remove = {
            let conns = self.connections.read().await;
            conns
                .keys()
                .filter(|id| !new_node_ids.contains(id))
                .cloned()
                .collect::<Vec<_>>()
        };

        for node_id in nodes_to_remove {
            self.disconnect(node_id).await;
        }

        // 2. 建立新连接或更新已有连接的代理列表
        for group in server_groups {
            let new_proxy_ids: HashSet<i64> = group.proxies.iter().map(|p| p.proxy_id).collect();

            let needs_connect = {
                let conns = self.connections.read().await;
                match conns.get(&group.node_id) {
                    Some(conn) => {
                        // 已有连接，更新代理列表（暂不需要重连）
                        if conn.proxy_ids != new_proxy_ids {
                            info!(
                                "节点 #{} 代理列表已更新: {} -> {} 个",
                                group.node_id,
                                conn.proxy_ids.len(),
                                new_proxy_ids.len()
                            );
                        }
                        false
                    }
                    None => true,
                }
            };

            if needs_connect {
                self.connect(group, new_proxy_ids).await;
            } else {
                // 更新代理列表
                let mut conns = self.connections.write().await;
                if let Some(conn) = conns.get_mut(&group.node_id) {
                    conn.proxy_ids = new_proxy_ids;
                }
            }
        }
    }

    /// 建立到指定 Server 的连接
    async fn connect(&self, group: ServerProxyGroup, proxy_ids: HashSet<i64>) {
        let node_id = group.node_id;
        let server_addr_str = format!("{}:{}", group.server_addr, group.server_port);
        let server_addr: SocketAddr = match server_addr_str.parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("节点 #{} 地址无效 ({}): {}", node_id, server_addr_str, e);
                return;
            }
        };

        info!(
            "连接到节点 #{} ({}), 协议: {:?}, 代理数: {}",
            node_id, server_addr, group.protocol, proxy_ids.len()
        );

        let token = self.token.clone();
        let log_collector = self.log_collector.clone();
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let cancel_clone = cancel_token.clone();
        let protocol = group.protocol.clone();
        let kcp_config = group.kcp.clone();

        let handle = tokio::spawn(async move {
            loop {
                // 创建连接器
                let connector: Arc<dyn TunnelConnector> = match protocol {
                    TunnelProtocol::Quic => {
                        match QuicConnector::new() {
                            Ok(c) => Arc::new(c),
                            Err(e) => {
                                error!("节点 #{} 创建 QUIC 连接器失败: {}", node_id, e);
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                continue;
                            }
                        }
                    }
                    TunnelProtocol::Kcp => {
                        Arc::new(KcpConnector::new(kcp_config.clone()))
                    }
                };

                // 连接并保持
                tokio::select! {
                    result = connector::connect_once(
                        connector,
                        server_addr,
                        &token,
                        log_collector.clone(),
                    ) => {
                        match result {
                            Ok(_) => info!("节点 #{} 连接已关闭", node_id),
                            Err(e) => error!("节点 #{} 连接错误: {}", node_id, e),
                        }
                    }
                    _ = cancel_clone.cancelled() => {
                        info!("节点 #{} 连接已取消", node_id);
                        return;
                    }
                }

                // 检查是否已取消
                if cancel_clone.is_cancelled() {
                    return;
                }

                warn!("节点 #{} 连接断开，5秒后重连...", node_id);
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
                    _ = cancel_clone.cancelled() => {
                        info!("节点 #{} 重连已取消", node_id);
                        return;
                    }
                }
            }
        });

        let conn = ServerConnection {
            node_id,
            proxy_ids,
            cancel_token,
            handle,
        };

        let mut conns = self.connections.write().await;
        conns.insert(node_id, conn);
    }

    /// 断开指定节点的连接
    async fn disconnect(&self, node_id: i64) {
        let conn = {
            let mut conns = self.connections.write().await;
            conns.remove(&node_id)
        };

        if let Some(conn) = conn {
            info!("断开节点 #{} 连接", node_id);
            conn.cancel_token.cancel();
            // 不等待 handle 完成，让它自行退出
        }
    }
}
