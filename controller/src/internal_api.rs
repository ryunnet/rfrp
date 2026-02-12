//! Controller 内部 API
//!
//! 暴露给 frps 调用的内部端点：
//! - POST /internal/auth/validate-token  - 验证客户端 token
//! - POST /internal/clients/{id}/online  - 上报客户端上下线
//! - POST /internal/traffic/report       - 批量上报流量数据
//! - GET  /internal/clients/{id}/proxies - 获取客户端代理配置
//! - GET  /internal/traffic/check-limit/{client_id} - 检查流量限制
//! - POST /internal/nodes/register       - 节点自注册

use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{ConnectInfo, Extension, Path},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use chrono::Utc;

use common::protocol::auth::{
    ClientAuthProvider, TrafficLimitResponse, ValidateTokenResponse,
};
use common::protocol::control::ProxyConfig;
use common::protocol::traffic::TrafficReportRequest;
use common::protocol::node_register::{NodeRegisterRequest, NodeRegisterResponse};

use crate::config::Config;
use crate::config_manager::ConfigManager;
use crate::node_manager::NodeManager;
use crate::entity::{Client, Node, Proxy, User, UserClient, client, node, proxy, user, user_client};
use crate::migration::get_connection;

/// 本地认证提供者（Controller 直接查询数据库）
pub struct LocalControllerAuthProvider;

impl LocalControllerAuthProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ClientAuthProvider for LocalControllerAuthProvider {
    async fn validate_token(&self, token: &str) -> Result<ValidateTokenResponse> {
        let db = get_connection().await;

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

        // 检查流量限制
        let user_clients = UserClient::find()
            .filter(user_client::Column::ClientId.eq(client_id))
            .all(db)
            .await
            .unwrap_or_default();

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

/// 内部 API 状态
#[derive(Clone)]
struct InternalState {
    config: Arc<Config>,
    config_manager: Arc<ConfigManager>,
    node_manager: Arc<NodeManager>,
}

/// 验证内部 API 密钥
fn verify_internal_secret(headers: &HeaderMap, expected_secret: &str) -> bool {
    if expected_secret.is_empty() {
        return true; // 未配置密钥时跳过验证
    }
    headers
        .get("X-Internal-Secret")
        .and_then(|v| v.to_str().ok())
        .map(|s| s == expected_secret)
        .unwrap_or(false)
}

// === 内部 API 处理函数 ===

#[derive(Deserialize)]
struct ValidateTokenRequest {
    token: String,
}

async fn handle_validate_token(
    Extension(state): Extension<InternalState>,
    headers: HeaderMap,
    Json(req): Json<ValidateTokenRequest>,
) -> impl IntoResponse {
    if !verify_internal_secret(&headers, &state.config.get_internal_secret()) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid secret"})));
    }

    let provider = LocalControllerAuthProvider::new();
    match provider.validate_token(&req.token).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize)]
struct ClientOnlineRequest {
    online: bool,
}

async fn handle_client_online(
    Extension(state): Extension<InternalState>,
    headers: HeaderMap,
    Path(client_id): Path<i64>,
    Json(req): Json<ClientOnlineRequest>,
) -> impl IntoResponse {
    if !verify_internal_secret(&headers, &state.config.get_internal_secret()) {
        return StatusCode::UNAUTHORIZED;
    }

    let provider = LocalControllerAuthProvider::new();
    match provider.set_client_online(client_id, req.online).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn handle_traffic_report(
    Extension(state): Extension<InternalState>,
    headers: HeaderMap,
    Json(req): Json<TrafficReportRequest>,
) -> impl IntoResponse {
    if !verify_internal_secret(&headers, &state.config.get_internal_secret()) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid secret"})));
    }

    // 处理流量上报 - 使用 TrafficManager 记录
    let traffic_manager = crate::traffic::TrafficManager::new();
    for record in req.records {
        let client_id = record.client_id.parse::<i64>().unwrap_or(0);
        traffic_manager.record_traffic(
            record.proxy_id,
            client_id,
            record.user_id,
            record.bytes_sent,
            record.bytes_received,
        ).await;
    }

    (StatusCode::OK, Json(serde_json::json!({"success": true})))
}

async fn handle_get_client_proxies(
    Extension(state): Extension<InternalState>,
    headers: HeaderMap,
    Path(client_id): Path<i64>,
) -> impl IntoResponse {
    if !verify_internal_secret(&headers, &state.config.get_internal_secret()) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid secret"})));
    }

    let provider = LocalControllerAuthProvider::new();
    match provider.get_client_proxies(client_id).await {
        Ok(proxies) => (StatusCode::OK, Json(serde_json::to_value(proxies).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn handle_check_traffic_limit(
    Extension(state): Extension<InternalState>,
    headers: HeaderMap,
    Path(client_id): Path<i64>,
) -> impl IntoResponse {
    if !verify_internal_secret(&headers, &state.config.get_internal_secret()) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid secret"})));
    }

    let provider = LocalControllerAuthProvider::new();
    match provider.check_traffic_limit(client_id).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

/// 处理节点自注册
async fn handle_node_register(
    Extension(state): Extension<InternalState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<NodeRegisterRequest>,
) -> impl IntoResponse {
    let db = get_connection().await;

    // 用 token 匹配 node.secret
    let node_model = match Node::find()
        .filter(node::Column::Secret.eq(&req.token))
        .one(db)
        .await
    {
        Ok(Some(n)) => n,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                "error": "无效的 token，未找到匹配的节点"
            })));
        }
        Err(e) => {
            error!("查询节点失败: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": e.to_string()
            })));
        }
    };

    let node_id = node_model.id;
    let node_name = node_model.name.clone();

    // 从 peer address 构建内部 API URL
    let peer_ip = addr.ip();
    let internal_url = format!("http://{}:{}", peer_ip, req.internal_port);

    // 更新节点信息
    let mut active: node::ActiveModel = node_model.into();
    active.url = Set(internal_url.clone());
    active.tunnel_port = Set(req.tunnel_port as i32);
    active.tunnel_protocol = Set(req.tunnel_protocol.clone());
    active.is_online = Set(true);
    active.updated_at = Set(Utc::now().naive_utc());

    if let Err(e) = active.update(db).await {
        error!("更新节点 #{} 失败: {}", node_id, e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        })));
    }

    // 动态加载到 NodeManager
    state.node_manager.add_node(node_id, internal_url.clone(), req.token.clone()).await;

    // 构建 controller 内部 API 地址
    let controller_internal_url = format!("http://{}:{}", addr.ip(), state.config.internal_port);

    info!("节点 #{} ({}) 已注册，内部 API: {}", node_id, node_name, internal_url);

    (StatusCode::OK, Json(serde_json::to_value(NodeRegisterResponse {
        node_id,
        node_name,
        internal_secret: state.config.get_internal_secret(),
        controller_internal_url,
    }).unwrap()))
}

/// 启动内部 API 服务
pub fn start_internal_api(
    config: Arc<Config>,
    config_manager: Arc<ConfigManager>,
    node_manager: Arc<NodeManager>,
) -> tokio::task::JoinHandle<()> {
    let state = InternalState {
        config: config.clone(),
        config_manager,
        node_manager,
    };

    let internal_port = config.internal_port;

    tokio::spawn(async move {
        let app = Router::new()
            .route("/internal/auth/validate-token", post(handle_validate_token))
            .route("/internal/clients/{id}/online", post(handle_client_online))
            .route("/internal/traffic/report", post(handle_traffic_report))
            .route("/internal/clients/{id}/proxies", get(handle_get_client_proxies))
            .route("/internal/traffic/check-limit/{client_id}", get(handle_check_traffic_limit))
            .route("/internal/nodes/register", post(handle_node_register))
            .layer(Extension(state));

        let addr = format!("0.0.0.0:{}", internal_port);
        info!("🔗 内部 API 服务启动: http://{}", addr);

        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                error!("内部 API 服务绑定失败: {}", e);
                return;
            }
        };

        if let Err(e) = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await {
            error!("内部 API 服务错误: {}", e);
        }
    })
}
