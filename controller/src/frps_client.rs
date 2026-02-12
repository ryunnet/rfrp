//! frps 远程代理控制客户端
//!
//! 通过 HTTP REST API 调用 frps 的内部接口，
//! 实现 ProxyControl trait。

use anyhow::Result;
use async_trait::async_trait;
use common::protocol::control::{
    ConnectedClient, LogEntry, ProxyControl, ServerStatus,
};
use tracing::{debug, error};

/// 远程代理控制客户端
pub struct RemoteProxyControl {
    base_url: String,
    secret: String,
    client: reqwest::Client,
}

impl RemoteProxyControl {
    pub fn new(base_url: String, secret: String) -> Self {
        Self {
            base_url,
            secret,
            client: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

#[async_trait]
impl ProxyControl for RemoteProxyControl {
    async fn start_proxy(&self, client_id: &str, proxy_id: i64) -> Result<()> {
        let url = self.url("/internal/proxy/start");
        debug!("调用 frps 启动代理: client_id={}, proxy_id={}", client_id, proxy_id);

        let resp = self.client
            .post(&url)
            .header("X-Internal-Secret", &self.secret)
            .json(&serde_json::json!({
                "client_id": client_id,
                "proxy_id": proxy_id,
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("frps 启动代理失败: {} - {}", status, body))
        }
    }

    async fn stop_proxy(&self, client_id: &str, proxy_id: i64) -> Result<()> {
        let url = self.url("/internal/proxy/stop");
        debug!("调用 frps 停止代理: client_id={}, proxy_id={}", client_id, proxy_id);

        let resp = self.client
            .post(&url)
            .header("X-Internal-Secret", &self.secret)
            .json(&serde_json::json!({
                "client_id": client_id,
                "proxy_id": proxy_id,
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("frps 停止代理失败: {} - {}", status, body))
        }
    }

    async fn get_connected_clients(&self) -> Result<Vec<ConnectedClient>> {
        let url = self.url("/internal/status");
        debug!("调用 frps 获取连接状态");

        let resp = self.client
            .get(&url)
            .header("X-Internal-Secret", &self.secret)
            .send()
            .await?;

        if resp.status().is_success() {
            let status: ServerStatus = resp.json().await?;
            Ok(status.connected_clients)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("frps 获取状态失败: {} - {}", status, body);
            Ok(vec![])
        }
    }

    async fn fetch_client_logs(&self, client_id: &str, count: u16) -> Result<Vec<LogEntry>> {
        let url = self.url(&format!("/internal/client/{}/logs", client_id));
        debug!("调用 frps 获取客户端日志: client_id={}", client_id);

        let resp = self.client
            .post(&url)
            .header("X-Internal-Secret", &self.secret)
            .json(&serde_json::json!({
                "count": count,
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            let logs: Vec<LogEntry> = resp.json().await?;
            Ok(logs)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("frps 获取日志失败: {} - {}", status, body);
            Ok(vec![])
        }
    }

    async fn get_server_status(&self) -> Result<ServerStatus> {
        let url = self.url("/internal/status");
        debug!("调用 frps 获取服务器状态");

        let resp = self.client
            .get(&url)
            .header("X-Internal-Secret", &self.secret)
            .send()
            .await?;

        if resp.status().is_success() {
            let status: ServerStatus = resp.json().await?;
            Ok(status)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("frps 获取状态失败: {} - {}", status, body))
        }
    }
}
