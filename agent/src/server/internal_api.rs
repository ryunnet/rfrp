//! Agent Server 内部 API（供 Controller 调用）
//!
//! 在 controller 模式下，agent server 暴露以下端点供 controller 的 RemoteProxyControl 调用：
//! - POST /internal/proxy/start  - 启动代理监听器
//! - POST /internal/proxy/stop   - 停止代理监听器
//! - GET  /internal/status       - 获取服务器状态
//! - POST /internal/client/{id}/logs - 获取客户端日志

use std::sync::Arc;
use axum::{
    extract::{Extension, Path},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tracing::{error, info};

use common::protocol::control::{
    ProxyControl,
    StartProxyRequest, StopProxyRequest,
};
use common::protocol::auth::ClientAuthProvider;

use crate::server::proxy_server::{ConnectionProvider, ProxyListenerManager};

/// 内部 API 状态
#[derive(Clone)]
struct InternalApiState {
    secret: String,
    proxy_control: Arc<dyn ProxyControl>,
    auth_provider: Arc<dyn ClientAuthProvider>,
    listener_manager: Arc<ProxyListenerManager>,
    conn_provider: ConnectionProvider,
}

/// 验证内部 API 密钥
fn verify_secret(headers: &HeaderMap, expected: &str) -> bool {
    if expected.is_empty() {
        return true;
    }
    headers
        .get("X-Internal-Secret")
        .and_then(|v| v.to_str().ok())
        .map(|s| s == expected)
        .unwrap_or(false)
}

/// 启动代理
async fn handle_start_proxy(
    Extension(state): Extension<InternalApiState>,
    headers: HeaderMap,
    Json(req): Json<StartProxyRequest>,
) -> impl IntoResponse {
    if !verify_secret(&headers, &state.secret) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid secret"})));
    }

    // 从 controller 获取该客户端的代理配置列表
    let client_id_num: i64 = match req.client_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid client_id"})),
            );
        }
    };
    match state.auth_provider.get_client_proxies(client_id_num).await {
        Ok(proxies) => {
            // 找到目标代理
            let target_proxies: Vec<_> = proxies.into_iter()
                .filter(|p| p.proxy_id == req.proxy_id)
                .collect();

            if target_proxies.is_empty() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "proxy not found"})),
                );
            }

            match state.listener_manager.start_client_proxies_from_configs(
                req.client_id.clone(),
                target_proxies,
                state.conn_provider.clone(),
            ).await {
                Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))),
                Err(e) => {
                    error!("启动代理失败: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": e.to_string()})),
                    )
                }
            }
        }
        Err(e) => {
            error!("获取代理配置失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

/// 停止代理
async fn handle_stop_proxy(
    Extension(state): Extension<InternalApiState>,
    headers: HeaderMap,
    Json(req): Json<StopProxyRequest>,
) -> impl IntoResponse {
    if !verify_secret(&headers, &state.secret) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid secret"})));
    }

    match state.proxy_control.stop_proxy(&req.client_id, req.proxy_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))),
        Err(e) => {
            error!("停止代理失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

/// 获取服务器状态
async fn handle_status(
    Extension(state): Extension<InternalApiState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !verify_secret(&headers, &state.secret) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid secret"})));
    }

    match state.proxy_control.get_server_status().await {
        Ok(status) => (StatusCode::OK, Json(serde_json::to_value(status).unwrap())),
        Err(e) => {
            error!("获取状态失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

/// 获取客户端日志
#[derive(Deserialize)]
struct LogRequest {
    count: Option<u16>,
}

async fn handle_client_logs(
    Extension(state): Extension<InternalApiState>,
    headers: HeaderMap,
    Path(client_id): Path<String>,
    Json(req): Json<LogRequest>,
) -> impl IntoResponse {
    if !verify_secret(&headers, &state.secret) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid secret"})));
    }

    let count = req.count.unwrap_or(100);
    match state.proxy_control.fetch_client_logs(&client_id, count).await {
        Ok(logs) => (StatusCode::OK, Json(serde_json::to_value(logs).unwrap())),
        Err(e) => {
            error!("获取客户端日志失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

/// 启动 Agent 内部 API 服务（controller 模式下使用）
pub fn start_agent_internal_api(
    bind_port: u16,
    secret: String,
    proxy_control: Arc<dyn ProxyControl>,
    auth_provider: Arc<dyn ClientAuthProvider>,
    listener_manager: Arc<ProxyListenerManager>,
    conn_provider: ConnectionProvider,
) {
    let state = InternalApiState {
        secret,
        proxy_control,
        auth_provider,
        listener_manager,
        conn_provider,
    };

    let app = Router::new()
        .route("/internal/proxy/start", post(handle_start_proxy))
        .route("/internal/proxy/stop", post(handle_stop_proxy))
        .route("/internal/status", get(handle_status))
        .route("/internal/client/{id}/logs", post(handle_client_logs))
        .layer(Extension(state));

    let addr = format!("0.0.0.0:{}", bind_port);
    info!("Agent 内部 API 启动: {}", addr);

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}
