pub mod auth;
pub mod client;
pub mod proxy;
pub mod user;
pub mod traffic;
pub mod dashboard;
pub mod client_logs;

// Re-export common handler modules
pub use auth::*;
pub use client::*;
pub use proxy::*;
pub use user::*;
pub use traffic::*;
pub use dashboard::*;
pub use client_logs::*;

use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> axum::response::Json<Self> {
        axum::response::Json(Self {
            success: true,
            data: Some(data),
            message: "Success".to_string(),
        })
    }

    pub fn error(message: String) -> axum::response::Json<Self> {
        axum::response::Json(Self {
            success: false,
            data: None,
            message,
        })
    }
}
