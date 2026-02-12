use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, PaginatorTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::server::{
    auth::{generate_random_password, hash_password},
    entity::{User, UserClient},
    migration::get_connection,
    middleware::AuthUser,
};

use super::ApiResponse;

#[derive(Serialize)]
pub struct UserWithClientCount {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub created_at: String,
    pub updated_at: String,
    pub client_count: u64,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: Option<String>,
    pub is_admin: Option<bool>,
    pub upload_limit_gb: Option<f64>,
    pub download_limit_gb: Option<f64>,
    pub traffic_reset_cycle: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub is_admin: Option<bool>,
    pub upload_limit_gb: Option<f64>,
    pub download_limit_gb: Option<f64>,
    pub traffic_reset_cycle: Option<String>,
    pub is_traffic_exceeded: Option<bool>,
}

/// GET /api/users - Get all users (admin only)
pub async fn list_users(Extension(auth_user_opt): Extension<Option<AuthUser>>) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<Vec<UserWithClientCount>>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    match User::find().all(db).await {
        Ok(users) => {
            // Count clients for each user
            let mut users_with_count = Vec::new();
            for user in users {
                let client_count = match UserClient::find()
                    .filter(crate::server::entity::user_client::Column::UserId.eq(user.id))
                    .count(db)
                    .await
                {
                    Ok(count) => count,
                    Err(_) => 0,
                };

                users_with_count.push(UserWithClientCount {
                    id: user.id,
                    username: user.username,
                    is_admin: user.is_admin,
                    created_at: user.created_at.to_string(),
                    updated_at: user.updated_at.to_string(),
                    client_count,
                });
            }

            (StatusCode::OK, ApiResponse::success(users_with_count))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<Vec<UserWithClientCount>>::error(format!("Failed to list users: {}", e)),
        ),
    }
}

/// POST /api/users - Create a new user (admin only)
pub async fn create_user(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<serde_json::Value>::error("Not authenticated".to_string())),
    };
    // Check if username already exists
    let db = get_connection().await;
    match User::find()
        .filter(crate::server::entity::user::Column::Username.eq(&req.username))
        .one(db)
        .await
    {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                ApiResponse::<serde_json::Value>::error("Username already exists".to_string()),
            )
        }
        Ok(None) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<serde_json::Value>::error(format!("Failed to check username: {}", e)),
            )
        }
    };

    // Hash password or generate random one
    let password = req.password.clone().unwrap_or_else(|| generate_random_password(16));
    let password_hash = match hash_password(&password) {
        Ok(hash) => hash,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<serde_json::Value>::error(format!("Failed to hash password: {}", e)),
            )
        }
    };

    // Create user
    let now = Utc::now().naive_utc();
    let new_user = crate::server::entity::user::ActiveModel {
        id: NotSet,
        username: Set(req.username),
        password_hash: Set(password_hash),
        is_admin: Set(req.is_admin.unwrap_or(false)),
        total_bytes_sent: Set(0),
        total_bytes_received: Set(0),
        upload_limit_gb: Set(req.upload_limit_gb),
        download_limit_gb: Set(req.download_limit_gb),
        traffic_reset_cycle: Set(req.traffic_reset_cycle.unwrap_or_else(|| "none".to_string())),
        last_reset_at: Set(None),
        is_traffic_exceeded: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
    };

    match new_user.insert(db).await {
        Ok(user) => {
            // Log generated password if random
            if req.password.is_none() {
                tracing::info!("Generated password for user '{}': {}", user.username, password);
            }

            // Return user without password hash
            let user_response = serde_json::json!({
                "id": user.id,
                "username": user.username,
                "is_admin": user.is_admin,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
                "generated_password": if req.password.is_none() { Some(password) } else { None },
            });

            (StatusCode::OK, ApiResponse::<serde_json::Value>::success(user_response))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<serde_json::Value>::error(format!("Failed to create user: {}", e)),
        ),
    }
}

/// PUT /api/users/:id - Update a user (admin only)
pub async fn update_user(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<serde_json::Value>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    // Find user
    let user = match User::find_by_id(id).one(db).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                ApiResponse::<serde_json::Value>::error("User not found".to_string()),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<serde_json::Value>::error(format!("Failed to find user: {}", e)),
            )
        }
    };

    let mut user: crate::server::entity::user::ActiveModel = user.into();

    // Check if new username conflicts
    if let Some(new_username) = &req.username {
        match User::find()
            .filter(crate::server::entity::user::Column::Username.eq(new_username))
            .filter(crate::server::entity::user::Column::Id.ne(id))
            .one(db)
            .await
        {
            Ok(Some(_)) => {
                return (
                    StatusCode::CONFLICT,
                    ApiResponse::<serde_json::Value>::error(
                        "Username already exists".to_string(),
                    ),
                )
            }
            Ok(None) => {
                user.username = Set(new_username.clone());
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<serde_json::Value>::error(format!(
                        "Failed to check username: {}",
                        e
                    )),
                )
            }
        }
    }

    // Update password if provided
    if let Some(password) = &req.password {
        let password_hash = match hash_password(password) {
            Ok(hash) => hash,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<serde_json::Value>::error(format!(
                        "Failed to hash password: {}",
                        e
                    )),
                )
            }
        };
        user.password_hash = Set(password_hash);
    }

    // Update admin status if provided
    if let Some(is_admin) = req.is_admin {
        user.is_admin = Set(is_admin);
    }

    // Update traffic limits if provided
    if let Some(upload_limit) = req.upload_limit_gb {
        user.upload_limit_gb = Set(Some(upload_limit));
    }
    if let Some(download_limit) = req.download_limit_gb {
        user.download_limit_gb = Set(Some(download_limit));
    }
    if let Some(cycle) = req.traffic_reset_cycle {
        user.traffic_reset_cycle = Set(cycle);
    }
    if let Some(exceeded) = req.is_traffic_exceeded {
        user.is_traffic_exceeded = Set(exceeded);
    }

    user.updated_at = Set(Utc::now().naive_utc());

    match user.update(db).await {
        Ok(updated) => {
            let user_response = serde_json::json!({
                "id": updated.id,
                "username": updated.username,
                "is_admin": updated.is_admin,
                "created_at": updated.created_at,
                "updated_at": updated.updated_at,
            });

            (StatusCode::OK, ApiResponse::<serde_json::Value>::success(user_response))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<serde_json::Value>::error(format!("Failed to update user: {}", e)),
        ),
    }
}

/// DELETE /api/users/:id - Delete a user (admin only)
pub async fn delete_user(Extension(auth_user_opt): Extension<Option<AuthUser>>, Path(id): Path<i64>) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<&str>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    match User::delete_by_id(id).exec(db).await {
        Ok(_) => (StatusCode::OK, ApiResponse::success("User deleted successfully")),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(format!("Failed to delete user: {}", e)),
        ),
    }
}

/// GET /api/users/:id/clients - Get user's client list (admin only)
pub async fn get_user_clients(Extension(auth_user_opt): Extension<Option<AuthUser>>, Path(user_id): Path<i64>) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<Vec<crate::server::entity::client::Model>>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    match UserClient::find()
        .filter(crate::server::entity::user_client::Column::UserId.eq(user_id))
        .find_also_related(crate::server::entity::Client)
        .all(db)
        .await
    {
        Ok(user_clients) => {
            let clients: Vec<_> = user_clients
                .into_iter()
                .filter_map(|(_, client)| client)
                .collect();

            (StatusCode::OK, ApiResponse::success(clients))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<Vec<crate::server::entity::client::Model>>::error(format!(
                "Failed to get user clients: {}",
                e
            )),
        ),
    }
}

/// POST /api/users/:id/clients/:client_id - Assign client to user (admin only)
pub async fn assign_client_to_user(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Path((user_id, client_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<&str>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    // Check if user exists
    match User::find_by_id(user_id).one(db).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                ApiResponse::<&str>::error("User not found".to_string()),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<&str>::error(format!("Failed to find user: {}", e)),
            )
        }
    };

    // Check if client exists
    match crate::server::entity::Client::find_by_id(client_id).one(db).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                ApiResponse::<&str>::error("Client not found".to_string()),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<&str>::error(format!("Failed to find client: {}", e)),
            )
        }
    };

    // Check if already assigned
    match UserClient::find()
        .filter(crate::server::entity::user_client::Column::UserId.eq(user_id))
        .filter(crate::server::entity::user_client::Column::ClientId.eq(client_id))
        .one(db)
        .await
    {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                ApiResponse::<&str>::error("Client already assigned to user".to_string()),
            )
        }
        Ok(None) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<&str>::error(format!("Failed to check assignment: {}", e)),
            )
        }
    };

    // Create assignment
    let now = Utc::now().naive_utc();
    let new_user_client = crate::server::entity::user_client::ActiveModel {
        id: NotSet,
        user_id: Set(user_id),
        client_id: Set(client_id),
        created_at: Set(now),
    };

    match new_user_client.insert(db).await {
        Ok(_) => (StatusCode::OK, ApiResponse::success("Client assigned successfully")),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(format!("Failed to assign client: {}", e)),
        ),
    }
}

/// DELETE /api/users/:id/clients/:client_id - Remove client from user (admin only)
pub async fn remove_client_from_user(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Path((user_id, client_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let _auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<&str>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    match UserClient::delete_many()
        .filter(crate::server::entity::user_client::Column::UserId.eq(user_id))
        .filter(crate::server::entity::user_client::Column::ClientId.eq(client_id))
        .exec(db)
        .await
    {
        Ok(result) => {
            if result.rows_affected > 0 {
                (
                    StatusCode::OK,
                    ApiResponse::success("Client removed successfully"),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    ApiResponse::<&str>::error("Assignment not found".to_string()),
                )
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(format!("Failed to remove client: {}", e)),
        ),
    }
}
