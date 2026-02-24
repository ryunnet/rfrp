use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::{
    entity::{Subscription, UserSubscription, User},
    migration::get_connection,
    middleware::AuthUser,
};

use super::ApiResponse;

#[derive(Serialize)]
pub struct UserSubscriptionWithDetails {
    pub id: i64,
    #[serde(rename = "userId")]
    pub user_id: i64,
    #[serde(rename = "subscriptionId")]
    pub subscription_id: i64,
    #[serde(rename = "subscriptionName")]
    pub subscription_name: String,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
    #[serde(rename = "trafficQuotaGb")]
    pub traffic_quota_gb: f64,
    #[serde(rename = "trafficUsedGb")]
    pub traffic_used_gb: f64,
    #[serde(rename = "trafficRemainingGb")]
    pub traffic_remaining_gb: f64,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "isExpired")]
    pub is_expired: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateUserSubscriptionRequest {
    pub user_id: i64,
    pub subscription_id: i64,
    pub start_date: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserSubscriptionRequest {
    pub is_active: Option<bool>,
    pub traffic_used_gb: Option<f64>,
}

/// GET /api/user-subscriptions - 获取所有用户订阅（管理员）
pub async fn list_user_subscriptions(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<Vec<UserSubscriptionWithDetails>>::error("未认证".to_string()),
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

    match UserSubscription::find().all(db).await {
        Ok(user_subscriptions) => {
            let mut result = Vec::new();
            for us in user_subscriptions {
                if let Ok(Some(subscription)) = Subscription::find_by_id(us.subscription_id).one(db).await {
                    let now = Utc::now().naive_utc();
                    let is_expired = us.end_date < now;
                    let traffic_remaining_gb = (us.traffic_quota_gb - us.traffic_used_gb).max(0.0);

                    result.push(UserSubscriptionWithDetails {
                        id: us.id,
                        user_id: us.user_id,
                        subscription_id: us.subscription_id,
                        subscription_name: subscription.name,
                        start_date: us.start_date.to_string(),
                        end_date: us.end_date.to_string(),
                        traffic_quota_gb: us.traffic_quota_gb,
                        traffic_used_gb: us.traffic_used_gb,
                        traffic_remaining_gb,
                        is_active: us.is_active,
                        is_expired,
                        created_at: us.created_at.to_string(),
                        updated_at: us.updated_at.to_string(),
                    });
                }
            }
            (StatusCode::OK, ApiResponse::success(result))
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("获取用户订阅列表失败: {}", err)),
        ),
    }
}

/// GET /api/users/{user_id}/subscriptions - 获取指定用户的订阅
pub async fn get_user_subscriptions(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<Vec<UserSubscriptionWithDetails>>::error("未认证".to_string()),
            )
        }
    };

    // 用户只能查看自己的订阅，管理员可以查看所有用户的订阅
    if !auth_user.is_admin && auth_user.id != user_id {
        return (
            StatusCode::FORBIDDEN,
            ApiResponse::error("无权查看其他用户的订阅".to_string()),
        );
    }

    let db = get_connection().await;

    match UserSubscription::find()
        .filter(crate::entity::user_subscription::Column::UserId.eq(user_id))
        .all(db)
        .await
    {
        Ok(user_subscriptions) => {
            let mut result = Vec::new();
            for us in user_subscriptions {
                if let Ok(Some(subscription)) = Subscription::find_by_id(us.subscription_id).one(db).await {
                    let now = Utc::now().naive_utc();
                    let is_expired = us.end_date < now;
                    let traffic_remaining_gb = (us.traffic_quota_gb - us.traffic_used_gb).max(0.0);

                    result.push(UserSubscriptionWithDetails {
                        id: us.id,
                        user_id: us.user_id,
                        subscription_id: us.subscription_id,
                        subscription_name: subscription.name,
                        start_date: us.start_date.to_string(),
                        end_date: us.end_date.to_string(),
                        traffic_quota_gb: us.traffic_quota_gb,
                        traffic_used_gb: us.traffic_used_gb,
                        traffic_remaining_gb,
                        is_active: us.is_active,
                        is_expired,
                        created_at: us.created_at.to_string(),
                        updated_at: us.updated_at.to_string(),
                    });
                }
            }
            (StatusCode::OK, ApiResponse::success(result))
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("获取用户订阅失败: {}", err)),
        ),
    }
}

/// GET /api/users/{user_id}/subscriptions/active - 获取用户的激活订阅
pub async fn get_user_active_subscription(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<Option<UserSubscriptionWithDetails>>::error("未认证".to_string()),
            )
        }
    };

    if !auth_user.is_admin && auth_user.id != user_id {
        return (
            StatusCode::FORBIDDEN,
            ApiResponse::error("无权查看其他用户的订阅".to_string()),
        );
    }

    let db = get_connection().await;
    let now = Utc::now().naive_utc();

    match UserSubscription::find()
        .filter(crate::entity::user_subscription::Column::UserId.eq(user_id))
        .filter(crate::entity::user_subscription::Column::IsActive.eq(true))
        .filter(crate::entity::user_subscription::Column::EndDate.gt(now))
        .one(db)
        .await
    {
        Ok(Some(us)) => {
            if let Ok(Some(subscription)) = Subscription::find_by_id(us.subscription_id).one(db).await {
                let is_expired = us.end_date < now;
                let traffic_remaining_gb = (us.traffic_quota_gb - us.traffic_used_gb).max(0.0);

                let result = UserSubscriptionWithDetails {
                    id: us.id,
                    user_id: us.user_id,
                    subscription_id: us.subscription_id,
                    subscription_name: subscription.name,
                    start_date: us.start_date.to_string(),
                    end_date: us.end_date.to_string(),
                    traffic_quota_gb: us.traffic_quota_gb,
                    traffic_used_gb: us.traffic_used_gb,
                    traffic_remaining_gb,
                    is_active: us.is_active,
                    is_expired,
                    created_at: us.created_at.to_string(),
                    updated_at: us.updated_at.to_string(),
                };
                (StatusCode::OK, ApiResponse::success(Some(result)))
            } else {
                (StatusCode::OK, ApiResponse::success(None))
            }
        }
        Ok(None) => (StatusCode::OK, ApiResponse::success(None)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("获取用户激活订阅失败: {}", err)),
        ),
    }
}

/// POST /api/user-subscriptions - 创建用户订阅（管理员）
pub async fn create_user_subscription(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Json(req): Json<CreateUserSubscriptionRequest>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<crate::entity::user_subscription::Model>::error("未认证".to_string()),
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

    // 验证用户存在
    if User::find_by_id(req.user_id).one(db).await.ok().flatten().is_none() {
        return (
            StatusCode::BAD_REQUEST,
            ApiResponse::error("用户不存在".to_string()),
        );
    }

    // 获取订阅套餐信息
    let subscription = match Subscription::find_by_id(req.subscription_id).one(db).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
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

    let now = Utc::now().naive_utc();
    let start_date = if let Some(start_date_str) = req.start_date {
        match chrono::NaiveDateTime::parse_from_str(&start_date_str, "%Y-%m-%d %H:%M:%S") {
            Ok(dt) => dt,
            Err(_) => now,
        }
    } else {
        now
    };

    // 计算结束日期
    let end_date = match subscription.duration_type.as_str() {
        "daily" => start_date + Duration::days(subscription.duration_value as i64),
        "weekly" => start_date + Duration::weeks(subscription.duration_value as i64),
        "monthly" => start_date + Duration::days(30 * subscription.duration_value as i64),
        "yearly" => start_date + Duration::days(365 * subscription.duration_value as i64),
        _ => start_date + Duration::days(30),
    };

    let user_subscription = crate::entity::user_subscription::ActiveModel {
        id: NotSet,
        user_id: Set(req.user_id),
        subscription_id: Set(req.subscription_id),
        start_date: Set(start_date),
        end_date: Set(end_date),
        traffic_quota_gb: Set(subscription.traffic_quota_gb),
        traffic_used_gb: Set(0.0),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };

    // 创建用户订阅
    let created_subscription = match user_subscription.insert(db).await {
        Ok(us) => us,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::error(format!("创建用户订阅失败: {}", err)),
            )
        }
    };

    // 将订阅流量加到用户的流量配额
    if let Ok(Some(user)) = User::find_by_id(req.user_id).one(db).await {
        let mut user: crate::entity::user::ActiveModel = user.into();
        let current_quota = user.traffic_quota_gb.clone().unwrap().unwrap_or(0.0);
        user.traffic_quota_gb = Set(Some(current_quota + subscription.traffic_quota_gb));
        user.updated_at = Set(now);
        let _ = user.update(db).await;
    }

    (StatusCode::CREATED, ApiResponse::success(created_subscription))
}

/// PUT /api/user-subscriptions/{id} - 更新用户订阅（管理员）
pub async fn update_user_subscription(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserSubscriptionRequest>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<crate::entity::user_subscription::Model>::error("未认证".to_string()),
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

    let user_subscription = match UserSubscription::find_by_id(id).one(db).await {
        Ok(Some(us)) => us,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                ApiResponse::error("用户订阅不存在".to_string()),
            )
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::error(format!("查询用户订阅失败: {}", err)),
            )
        }
    };

    let mut user_subscription: crate::entity::user_subscription::ActiveModel = user_subscription.into();

    if let Some(is_active) = req.is_active {
        user_subscription.is_active = Set(is_active);
    }
    if let Some(traffic_used_gb) = req.traffic_used_gb {
        user_subscription.traffic_used_gb = Set(traffic_used_gb);
    }

    user_subscription.updated_at = Set(Utc::now().naive_utc());

    match user_subscription.update(db).await {
        Ok(user_subscription) => (StatusCode::OK, ApiResponse::success(user_subscription)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("更新用户订阅失败: {}", err)),
        ),
    }
}

/// DELETE /api/user-subscriptions/{id} - 删除用户订阅（管理员）
pub async fn delete_user_subscription(
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

    match UserSubscription::delete_by_id(id).exec(db).await {
        Ok(_) => (StatusCode::OK, ApiResponse::success(())),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("删除用户订阅失败: {}", err)),
        ),
    }
}
