use axum::{
    extract::{Request, Extension},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::{jwt, AppState};

/// Current authenticated user information extracted from JWT
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
}

/// Extract bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(auth_header[7..].to_string())
}

impl AuthUser {
    /// Create AuthUser from headers
    pub fn from_headers(headers: &HeaderMap, jwt_secret: &str) -> Result<Self, StatusCode> {
        let token = extract_bearer_token(headers)?;
        let claims = jwt::verify_token(&token, jwt_secret)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthUser {
            id: claims.sub,
            username: claims.username,
            is_admin: claims.is_admin,
        })
    }
}

/// Middleware to extract and store AuthUser in request extensions
pub async fn auth_middleware(
    Extension(app_state): Extension<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let jwt_secret = app_state.config.get_jwt_secret().unwrap_or_default();
    let auth_user = AuthUser::from_headers(request.headers(), &jwt_secret).ok();
    let mut request = request;
    request.extensions_mut().insert(auth_user);
    next.run(request).await
}
