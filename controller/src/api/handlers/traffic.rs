use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use super::ApiResponse;
use crate::middleware::AuthUser;
use crate::traffic::{get_traffic_overview, TrafficOverview};

#[derive(Debug, Deserialize)]
pub struct TrafficQuery {
    pub days: Option<i64>,
}

/// 获取流量总览
pub async fn get_traffic_overview_handler(
    Extension(auth_user): Extension<Option<AuthUser>>,
    Query(params): Query<TrafficQuery>,
) -> impl IntoResponse {
    // 验证用户是否已认证
    let auth_user = match auth_user {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<TrafficOverview>::error("未认证，请先登录".to_string()),
            )
        }
    };

    let days = params.days.unwrap_or(30);

    // 始终传入 user_id，在 get_traffic_overview 内部判断管理员权限
    match get_traffic_overview(Some(auth_user.id), days).await {
        Ok(overview) => (StatusCode::OK, ApiResponse::success(overview)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("获取流量统计失败: {}", e)),
        ),
    }
}

/// 获取指定用户的流量统计
pub async fn get_user_traffic_handler(
    Extension(auth_user): Extension<Option<AuthUser>>,
    Path(user_id): Path<i64>,
    Query(params): Query<TrafficQuery>,
) -> impl IntoResponse {
    // 验证用户是否已认证
    let auth_user = match auth_user {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<TrafficOverview>::error("未认证，请先登录".to_string()),
            )
        }
    };

    // 权限检查：只有管理员或用户本人可以查看
    if !auth_user.is_admin && auth_user.id != user_id {
        return (
            StatusCode::FORBIDDEN,
            ApiResponse::<TrafficOverview>::error("无权查看其他用户的流量统计".to_string()),
        );
    }

    let days = params.days.unwrap_or(30);

    match get_traffic_overview(Some(user_id), days).await {
        Ok(overview) => (StatusCode::OK, ApiResponse::success(overview)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("获取用户流量统计失败: {}", e)),
        ),
    }
}
