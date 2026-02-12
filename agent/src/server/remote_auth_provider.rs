//! 远程客户端认证提供者
//!
//! 通过 HTTP 调用 Controller 内部 API 实现 ClientAuthProvider trait。
//! 用于 agent server 以 controller 模式运行时。

use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, error};

use common::protocol::auth::{
    ClientAuthProvider, TrafficLimitResponse, ValidateTokenResponse,
};
use common::protocol::control::ProxyConfig;

/// 远程客户端认证提供者
pub struct RemoteClientAuthProvider {
    controller_internal_url: String,
    internal_secret: String,
    client: reqwest::Client,
}

impl RemoteClientAuthProvider {
    pub fn new(controller_internal_url: String, internal_secret: String) -> Self {
        Self {
            controller_internal_url,
            internal_secret,
            client: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.controller_internal_url, path)
    }
}

#[async_trait]
impl ClientAuthProvider for RemoteClientAuthProvider {
    async fn validate_token(&self, token: &str) -> Result<ValidateTokenResponse> {
        let url = self.url("/internal/auth/validate-token");
        debug!("远程验证 token");

        let resp = self.client
            .post(&url)
            .header("X-Internal-Secret", &self.internal_secret)
            .json(&serde_json::json!({ "token": token }))
            .send()
            .await?;

        if resp.status().is_success() {
            let result: ValidateTokenResponse = resp.json().await?;
            Ok(result)
        } else {
            let body = resp.text().await.unwrap_or_default();
            error!("远程验证 token 失败: {}", body);
            Ok(ValidateTokenResponse {
                client_id: 0,
                client_name: String::new(),
                allowed: false,
                reject_reason: Some(format!("Controller 验证失败: {}", body)),
            })
        }
    }

    async fn set_client_online(&self, client_id: i64, online: bool) -> Result<()> {
        let url = self.url(&format!("/internal/clients/{}/online", client_id));
        debug!("远程上报客户端 #{} 状态: online={}", client_id, online);

        let resp = self.client
            .post(&url)
            .header("X-Internal-Secret", &self.internal_secret)
            .json(&serde_json::json!({ "online": online }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            error!("远程上报客户端状态失败: {}", body);
        }

        Ok(())
    }

    async fn check_traffic_limit(&self, client_id: i64) -> Result<TrafficLimitResponse> {
        let url = self.url(&format!("/internal/traffic/check-limit/{}", client_id));
        debug!("远程检查客户端 #{} 流量限制", client_id);

        let resp = self.client
            .get(&url)
            .header("X-Internal-Secret", &self.internal_secret)
            .send()
            .await?;

        if resp.status().is_success() {
            let result: TrafficLimitResponse = resp.json().await?;
            Ok(result)
        } else {
            let body = resp.text().await.unwrap_or_default();
            error!("远程检查流量限制失败: {}", body);
            Ok(TrafficLimitResponse {
                exceeded: false,
                reason: None,
            })
        }
    }

    async fn get_client_proxies(&self, client_id: i64) -> Result<Vec<ProxyConfig>> {
        let url = self.url(&format!("/internal/clients/{}/proxies", client_id));
        debug!("远程获取客户端 #{} 代理配置", client_id);

        let resp = self.client
            .get(&url)
            .header("X-Internal-Secret", &self.internal_secret)
            .send()
            .await?;

        if resp.status().is_success() {
            let proxies: Vec<ProxyConfig> = resp.json().await?;
            Ok(proxies)
        } else {
            let body = resp.text().await.unwrap_or_default();
            error!("远程获取代理配置失败: {}", body);
            Ok(vec![])
        }
    }
}
