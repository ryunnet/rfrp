//! 本地客户端认证提供者实现
//!
//! 直接查询本地数据库进行认证，
//! 用于 Phase 0 阶段（单二进制模式）。

use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

use common::protocol::auth::{
    ClientAuthProvider, TrafficLimitResponse, ValidateTokenResponse,
};
use common::protocol::control::ProxyConfig;

use crate::server::entity::{Client, Proxy, User, UserClient, client, proxy, user_client};
use crate::server::migration::get_connection;

/// 本地客户端认证提供者
///
/// 直接查询 SQLite 数据库，无需网络通信。
pub struct LocalClientAuthProvider;

impl LocalClientAuthProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ClientAuthProvider for LocalClientAuthProvider {
    async fn validate_token(&self, token: &str) -> Result<ValidateTokenResponse> {
        let db = get_connection().await;

        // 查找对应的客户端
        let client = match Client::find()
            .filter(client::Column::Token.eq(token))
            .one(db)
            .await?
        {
            Some(c) => c,
            None => {
                return Ok(ValidateTokenResponse {
                    client_id: 0,
                    client_name: String::new(),
                    allowed: false,
                    reject_reason: Some("无效的 token".to_string()),
                });
            }
        };

        let client_id = client.id;
        let client_name = client.name.clone();

        // 检查该客户端绑定的用户是否有流量超限
        let user_clients = match UserClient::find()
            .filter(user_client::Column::ClientId.eq(client_id))
            .all(db)
            .await
        {
            Ok(ucs) => ucs,
            Err(e) => {
                error!("查询用户客户端关联失败: {}", e);
                return Ok(ValidateTokenResponse {
                    client_id,
                    client_name,
                    allowed: false,
                    reject_reason: Some(format!("查询用户关联失败: {}", e)),
                });
            }
        };

        // 检查所有关联用户的流量状态
        for uc in user_clients {
            if let Ok(Some(user)) = User::find_by_id(uc.user_id).one(db).await {
                if user.is_traffic_exceeded {
                    return Ok(ValidateTokenResponse {
                        client_id,
                        client_name,
                        allowed: false,
                        reject_reason: Some(format!(
                            "用户 {} (#{}) 流量已超限",
                            user.username, user.id
                        )),
                    });
                }
            }
        }

        Ok(ValidateTokenResponse {
            client_id,
            client_name,
            allowed: true,
            reject_reason: None,
        })
    }

    async fn set_client_online(&self, client_id: i64, online: bool) -> Result<()> {
        let db = get_connection().await;
        if let Some(client) = Client::find_by_id(client_id).one(db).await? {
            let mut client_active: client::ActiveModel = client.into();
            client_active.is_online = Set(online);
            debug!("更新客户端 #{} 状态: online={}", client_id, online);
            let _ = client_active.update(db).await;
        }
        Ok(())
    }

    async fn check_traffic_limit(&self, client_id: i64) -> Result<TrafficLimitResponse> {
        let db = get_connection().await;

        let user_clients = UserClient::find()
            .filter(user_client::Column::ClientId.eq(client_id))
            .all(db)
            .await?;

        for uc in user_clients {
            if let Ok(Some(user)) = User::find_by_id(uc.user_id).one(db).await {
                if user.is_traffic_exceeded {
                    return Ok(TrafficLimitResponse {
                        exceeded: true,
                        reason: Some(format!(
                            "用户 {} (#{}) 流量已超限",
                            user.username, user.id
                        )),
                    });
                }
            }
        }

        Ok(TrafficLimitResponse {
            exceeded: false,
            reason: None,
        })
    }

    async fn get_client_proxies(&self, client_id: i64) -> Result<Vec<ProxyConfig>> {
        let db = get_connection().await;
        let client_id_str = client_id.to_string();

        let proxies = Proxy::find()
            .filter(proxy::Column::ClientId.eq(&client_id_str))
            .filter(proxy::Column::Enabled.eq(true))
            .all(db)
            .await?;

        Ok(proxies
            .into_iter()
            .map(|p| ProxyConfig {
                proxy_id: p.id,
                client_id: p.client_id,
                name: p.name,
                proxy_type: p.proxy_type,
                local_ip: p.local_ip,
                local_port: p.local_port,
                remote_port: p.remote_port,
                enabled: p.enabled,
            })
            .collect())
    }
}
