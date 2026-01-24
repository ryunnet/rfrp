use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::{error, info};

use crate::{client_logs, middleware::AuthUser, AppState};

use super::ApiResponse;

/// GET /api/clients/{id}/logs - 获取客户端日志
pub async fn get_client_logs(
    Path(client_id): Path<i64>,
    Extension(_auth_user): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
) -> impl IntoResponse {
    info!("📋 请求客户端 {} 的日志", client_id);

    // 获取客户端连接
    let connections = app_state.proxy_server.get_client_connections();
    let conn = {
        let conns = connections.read().await;
        conns.get(&client_id.to_string()).cloned()
    };

    let conn = match conn {
        Some(c) => c,
        None => {
            return (
                StatusCode::NOT_FOUND,
                ApiResponse::<Vec<client_logs::LogEntry>>::error(
                    "客户端未连接或不在线".to_string(),
                ),
            )
        }
    };

    // 从客户端获取最近200条日志
    match client_logs::fetch_client_logs(conn, 200).await {
        Ok(logs) => {
            info!("✅ 成功获取客户端 {} 的 {} 条日志", client_id, logs.len());
            (StatusCode::OK, ApiResponse::success(logs))
        }
        Err(e) => {
            error!("❌ 获取客户端日志失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<Vec<client_logs::LogEntry>>::error(format!(
                    "获取日志失败: {}",
                    e
                )),
            )
        }
    }
}
