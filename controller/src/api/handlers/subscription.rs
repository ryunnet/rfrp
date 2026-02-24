use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, Set};
use serde::Deserialize;

use crate::{
    entity::Subscription,
    migration::get_connection,
    middleware::AuthUser,
};

use super::ApiResponse;

#[derive(Deserialize)]
pub struct CreateSubscriptionRequest {
    pub name: String,
    pub duration_type: String, // daily, weekly, monthly, yearly
    pub duration_value: Option<i32>,
    pub traffic_quota_gb: f64,
    pub price: Option<f64>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub name: Option<String>,
    pub duration_type: Option<String>,
    pub duration_value: Option<i32>,
    pub traffic_quota_gb: Option<f64>,
    pub price: Option<f64>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// GET /api/subscriptions - 获取所有订阅套餐
pub async fn list_subscriptions(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<Vec<crate::entity::subscription::Model>>::error(
                    "未认证".to_string(),
                ),
            )
        }
    };

    let db = get_connection().await;

    match Subscription::find().all(db).await {
        Ok(subscriptions) => (StatusCode::OK, ApiResponse::success(subscriptions)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("获取订阅列表失败: {}", err)),
        ),
    }
}

/// GET /api/subscriptions/active - 获取所有激活的订阅套餐
pub async fn list_active_subscriptions(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<Vec<crate::entity::subscription::Model>>::error(
                    "未认证".to_string(),
                ),
            )
        }
    };

    let db = get_connection().await;

    match Subscription::find()
        .filter(crate::entity::subscription::Column::IsActive.eq(true))
        .all(db)
        .await
    {
        Ok(subscriptions) => (StatusCode::OK, ApiResponse::success(subscriptions)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("获取激活订阅列表失败: {}", err)),
        ),
    }
}

/// GET /api/subscriptions/{id} - 获取单个订阅套餐
pub async fn get_subscription(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<crate::entity::subscription::Model>::error("未认证".to_string()),
            )
        }
    };

    let db = get_connection().await;

    match Subscription::find_by_id(id).one(db).await {
        Ok(Some(subscription)) => (StatusCode::OK, ApiResponse::success(subscription)),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            ApiResponse::error("订阅套餐不存在".to_string()),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("获取订阅套餐失败: {}", err)),
        ),
    }
}

/// POST /api/subscriptions - 创建订阅套餐（管理员）
pub async fn create_subscription(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<crate::entity::subscription::Model>::error("未认证".to_string()),
            )
        }
    };

    if !auth_user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            ApiResponse::error("需要管理员权限".to_string()),
        );
    }

    // 验证 duration_type
    if !["daily", "weekly", "monthly", "yearly"].contains(&req.duration_type.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            ApiResponse::error("无效的订阅周期类型".to_string()),
        );
    }

    let db = get_connection().await;
    let now = Utc::now().naive_utc();

    let subscription = crate::entity::subscription::ActiveModel {
        id: NotSet,
        name: Set(req.name),
        duration_type: Set(req.duration_type),
        duration_value: Set(req.duration_value.unwrap_or(1)),
        traffic_quota_gb: Set(req.traffic_quota_gb),
        price: Set(req.price),
        description: Set(req.description),
        is_active: Set(req.is_active.unwrap_or(true)),
        created_at: Set(now),
        updated_at: Set(now),
    };

    match subscription.insert(db).await {
        Ok(subscription) => (StatusCode::CREATED, ApiResponse::success(subscription)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("创建订阅套餐失败: {}", err)),
        ),
    }
}

/// PUT /api/subscriptions/{id} - 更新订阅套餐（管理员）
pub async fn update_subscription(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSubscriptionRequest>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<crate::entity::subscription::Model>::error("未认证".to_string()),
            )
        }
    };

    if !auth_user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            ApiResponse::error("需要管理员权限".to_string()),
        );
    }

    let db = get_connection().await;

    let subscription = match Subscription::find_by_id(id).one(db).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                ApiResponse::error("订阅套餐不存在".to_string()),
            )
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::error(format!("查询订阅套餐失败: {}", err)),
            )
        }
    };

    let mut subscription: crate::entity::subscription::ActiveModel = subscription.into();

    if let Some(name) = req.name {
        subscription.name = Set(name);
    }
    if let Some(duration_type) = req.duration_type {
        if !["daily", "weekly", "monthly", "yearly"].contains(&duration_type.as_str()) {
            return (
                StatusCode::BAD_REQUEST,
                ApiResponse::error("无效的订阅周期类型".to_string()),
            );
        }
        subscription.duration_type = Set(duration_type);
    }
    if let Some(duration_value) = req.duration_value {
        subscription.duration_value = Set(duration_value);
    }
    if let Some(traffic_quota_gb) = req.traffic_quota_gb {
        subscription.traffic_quota_gb = Set(traffic_quota_gb);
    }
    if let Some(price) = req.price {
        subscription.price = Set(Some(price));
    }
    if let Some(description) = req.description {
        subscription.description = Set(Some(description));
    }
    if let Some(is_active) = req.is_active {
        subscription.is_active = Set(is_active);
    }

    subscription.updated_at = Set(Utc::now().naive_utc());

    match subscription.update(db).await {
        Ok(subscription) => (StatusCode::OK, ApiResponse::success(subscription)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("更新订阅套餐失败: {}", err)),
        ),
    }
}

/// DELETE /api/subscriptions/{id} - 删除订阅套餐（管理员）
pub async fn delete_subscription(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<()>::error("未认证".to_string()),
            )
        }
    };

    if !auth_user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            ApiResponse::error("需要管理员权限".to_string()),
        );
    }

    let db = get_connection().await;

    match Subscription::delete_by_id(id).exec(db).await {
        Ok(_) => (StatusCode::OK, ApiResponse::success(())),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("删除订阅套餐失败: {}", err)),
        ),
    }
}
