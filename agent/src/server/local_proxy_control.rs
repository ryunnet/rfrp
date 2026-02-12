//! 本地代理控制实现
//!
//! 直接调用 ProxyListenerManager 和 ConnectionProvider，
//! 用于 Phase 0 阶段（单二进制模式）。

use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;
use std::collections::HashMap;

use common::protocol::control::{
    ConnectedClient, LogEntry, ProxyControl, ServerStatus,
};
use common::TunnelConnection;

use crate::server::proxy_server::{ConnectionProvider, ProxyListenerManager};
use crate::server::client_logs;

/// 本地代理控制实现
///
/// 直接调用 ProxyServer 内部组件，无需网络通信。
pub struct LocalProxyControl {
    listener_manager: Arc<ProxyListenerManager>,
    quic_connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    tunnel_connections: Arc<RwLock<HashMap<String, Arc<Box<dyn TunnelConnection>>>>>,
}

impl LocalProxyControl {
    pub fn new(
        listener_manager: Arc<ProxyListenerManager>,
        quic_connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
        tunnel_connections: Arc<RwLock<HashMap<String, Arc<Box<dyn TunnelConnection>>>>>,
    ) -> Self {
        Self {
            listener_manager,
            quic_connections,
            tunnel_connections,
        }
    }

    fn conn_provider(&self) -> ConnectionProvider {
        ConnectionProvider::new(
            self.quic_connections.clone(),
            self.tunnel_connections.clone(),
        )
    }
}

#[async_trait]
impl ProxyControl for LocalProxyControl {
    async fn start_proxy(&self, client_id: &str, proxy_id: i64) -> Result<()> {
        let conn_provider = self.conn_provider();
        self.listener_manager
            .start_single_proxy_unified(client_id.to_string(), proxy_id, conn_provider)
            .await
    }

    async fn stop_proxy(&self, client_id: &str, proxy_id: i64) -> Result<()> {
        self.listener_manager
            .stop_single_proxy(client_id, proxy_id)
            .await;
        Ok(())
    }

    async fn get_connected_clients(&self) -> Result<Vec<ConnectedClient>> {
        let mut clients = Vec::new();

        // QUIC connections
        {
            let conns = self.quic_connections.read().await;
            for (client_id, conn) in conns.iter() {
                clients.push(ConnectedClient {
                    client_id: client_id.clone(),
                    remote_address: conn.remote_address().to_string(),
                    protocol: "quic".to_string(),
                });
            }
        }

        // KCP/Tunnel connections
        {
            let conns = self.tunnel_connections.read().await;
            for (client_id, conn) in conns.iter() {
                clients.push(ConnectedClient {
                    client_id: client_id.clone(),
                    remote_address: conn.remote_address().to_string(),
                    protocol: "kcp".to_string(),
                });
            }
        }

        Ok(clients)
    }

    async fn fetch_client_logs(&self, client_id: &str, count: u16) -> Result<Vec<LogEntry>> {
        // 目前只支持 QUIC 连接获取日志
        let conn = {
            let conns = self.quic_connections.read().await;
            conns.get(client_id).cloned()
        };

        let conn = match conn {
            Some(c) => c,
            None => return Err(anyhow::anyhow!("客户端未连接或不在线")),
        };

        let logs = client_logs::fetch_client_logs(conn, count).await?;
        Ok(logs
            .into_iter()
            .map(|l| LogEntry {
                timestamp: l.timestamp.to_rfc3339(),
                level: l.level,
                message: l.message,
            })
            .collect())
    }

    async fn get_server_status(&self) -> Result<ServerStatus> {
        let clients = self.get_connected_clients().await?;
        let active_proxy_count = clients.len(); // 简化：用连接数近似
        Ok(ServerStatus {
            connected_clients: clients,
            active_proxy_count,
        })
    }
}
