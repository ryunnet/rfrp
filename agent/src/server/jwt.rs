use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,    // user id
    pub username: String,
    pub is_admin: bool,
    pub exp: i64,    // expiration time
    pub iat: i64,    // issued at
}

/// Generate a JWT token for a user
pub fn generate_token(user_id: i64, username: &str, is_admin: bool, jwt_secret: &str, expiration_hours: i64) -> Result<String> {
    let now = Utc::now();
    let expiration = now + Duration::hours(expiration_hours);

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        is_admin,
        iat: now.timestamp(),
        exp: expiration.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|e| anyhow!("Failed to generate token: {}", e))
}

/// Verify and decode a JWT token
pub fn verify_token(token: &str, jwt_secret: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| anyhow!("Failed to verify token: {}", e))
}
