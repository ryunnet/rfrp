use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{error, info};

use crate::server::{middleware::AuthUser, AppState};
use common::protocol::control::LogEntry;

use super::ApiResponse;

/// GET /api/clients/{id}/logs - 获取客户端日志
pub async fn get_client_logs(
    Path(client_id): Path<i64>,
    Extension(_auth_user): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
) -> impl IntoResponse {
    info!("请求客户端 {} 的日志", client_id);

    // 通过 ProxyControl trait 获取客户端日志
    match app_state.proxy_control.fetch_client_logs(&client_id.to_string(), 200).await {
        Ok(logs) => {
            info!("成功获取客户端 {} 的 {} 条日志", client_id, logs.len());
            (StatusCode::OK, ApiResponse::success(logs))
        }
        Err(e) => {
            error!("获取客户端日志失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<Vec<LogEntry>>::error(format!(
                    "获取日志失败: {}",
                    e
                )),
            )
        }
    }
}
