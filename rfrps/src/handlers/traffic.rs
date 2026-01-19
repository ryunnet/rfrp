use axum::{
    extract::{Path, Query},
    response::Json,
};
use serde::Deserialize;

use crate::handlers::ApiResponse;
use crate::traffic::{get_traffic_overview, TrafficOverview};

#[derive(Debug, Deserialize)]
pub struct TrafficQuery {
    pub days: Option<i64>,
}

/// 获取流量总览
pub async fn get_traffic_overview_handler(
    Query(params): Query<TrafficQuery>,
) -> Json<ApiResponse<TrafficOverview>> {
    let days = params.days.unwrap_or(30);

    // TODO: 从JWT中获取用户ID
    // 现在先返回所有数据（管理员视图）
    let user_id = None; // None表示管理员模式

    match get_traffic_overview(user_id, days).await {
        Ok(overview) => ApiResponse::success(overview),
        Err(e) => ApiResponse::error(format!("获取流量统计失败: {}", e)),
    }
}

/// 获取指定用户的流量统计
pub async fn get_user_traffic_handler(
    Path(user_id): Path<i64>,
    Query(params): Query<TrafficQuery>,
) -> Json<ApiResponse<TrafficOverview>> {
    let days = params.days.unwrap_or(30);

    match get_traffic_overview(Some(user_id), days).await {
        Ok(overview) => ApiResponse::success(overview),
        Err(e) => ApiResponse::error(format!("获取用户流量统计失败: {}", e)),
    }
}
