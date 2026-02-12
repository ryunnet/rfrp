use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::server::{
    auth::verify_password,
    entity::User,
    jwt::generate_token,
    middleware::AuthUser,
    migration::get_connection,
    AppState,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use super::ApiResponse;

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// POST /api/auth/login - User login
pub async fn login(
    Extension(app_state): Extension<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let db = get_connection().await;

    // Find user by username
    let user = match User::find()
        .filter(crate::server::entity::user::Column::Username.eq(&req.username))
        .one(db)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<LoginResponse>::error(
                    "Invalid username or password".to_string(),
                ),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<LoginResponse>::error(format!("Login failed: {}", e)),
            )
        }
    };

    // Verify password
    match verify_password(&req.password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::UNAUTHORIZED,
                ApiResponse::<LoginResponse>::error(
                    "Invalid username or password".to_string(),
                ),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<LoginResponse>::error(format!("Login failed: {}", e)),
            )
        }
    };

    // Get JWT secret from config
    let jwt_secret = match app_state.config.get_jwt_secret() {
        Ok(secret) => secret,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<LoginResponse>::error(format!("JWT configuration error: {}", e)),
            )
        }
    };

    // Generate JWT token
    let token = match generate_token(
        user.id,
        &user.username,
        user.is_admin,
        &jwt_secret,
        app_state.config.jwt_expiration_hours,
    ) {
        Ok(token) => token,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<LoginResponse>::error(format!("Failed to generate token: {}", e)),
            )
        }
    };

    let response = LoginResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
        },
    };

    (StatusCode::OK, ApiResponse::success(response))
}

/// GET /api/auth/me - Get current user info
pub async fn me(Extension(auth_user): Extension<Option<AuthUser>>) -> impl IntoResponse {
    let auth_user = match auth_user {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<UserInfo>::error("Not authenticated".to_string())),
    };
    let user_info = UserInfo {
        id: auth_user.id,
        username: auth_user.username,
        is_admin: auth_user.is_admin,
    };

    (StatusCode::OK, ApiResponse::success(user_info))
}
