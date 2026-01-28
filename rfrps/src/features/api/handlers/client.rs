use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::{entity::Client, migration::get_connection, middleware::AuthUser};

use super::ApiResponse;

#[derive(Deserialize)]
pub struct CreateClientRequest {
    pub name: String,
    pub token: Option<String>,
    pub upload_limit_gb: Option<f64>,
    pub download_limit_gb: Option<f64>,
    pub traffic_reset_cycle: Option<String>,
}

pub async fn list_clients(Extension(auth_user_opt): Extension<Option<AuthUser>>) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<Vec<crate::entity::client::Model>>::error("Not authenticated".to_string())),
    };

    let db = get_connection().await;

    let clients = if auth_user.is_admin {
        // Admin can see all clients
        match Client::find().all(db).await {
            Ok(clients) => clients,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<Vec<crate::entity::client::Model>>::error(format!(
                        "Failed to list clients: {}",
                        e
                    )),
                )
            }
        }
    } else {
        // Regular users can only see their assigned clients
        // First get the user's assigned client IDs
        let user_client_ids = match crate::entity::UserClient::find()
            .filter(crate::entity::user_client::Column::UserId.eq(auth_user.id))
            .all(db)
            .await
        {
            Ok(user_clients) => user_clients.into_iter().map(|uc| uc.client_id).collect::<Vec<_>>(),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<Vec<crate::entity::client::Model>>::error(format!(
                        "Failed to get user clients: {}",
                        e
                    )),
                )
            }
        };

        // If user has no assigned clients, return empty list
        if user_client_ids.is_empty() {
            vec![]
        } else {
            // Get clients for those IDs
            match Client::find()
                .filter(crate::entity::client::Column::Id.is_in(user_client_ids))
                .all(db)
                .await
            {
                Ok(clients) => clients,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ApiResponse::<Vec<crate::entity::client::Model>>::error(format!(
                            "Failed to list clients: {}",
                            e
                        )),
                    )
                }
            }
        }
    };

    (StatusCode::OK, ApiResponse::success(clients))
}

pub async fn create_client(
    Extension(_auth_user): Extension<Option<AuthUser>>,
    Json(req): Json<CreateClientRequest>,
) -> impl IntoResponse {
    let token = req.token.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = Utc::now().naive_utc();
    let new_client = crate::entity::client::ActiveModel {
        id: NotSet,
        name: Set(req.name),
        token: Set(token.clone()),
        is_online: NotSet,
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
    let db = get_connection().await;
    match new_client.insert(db).await {
        Ok(client) => (StatusCode::OK, ApiResponse::success(client)),
        Err(e) => {
            eprintln!("Failed to create client: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<crate::entity::client::Model>::error(format!(
                    "Failed to create client: {}",
                    e
                )),
            )
        }
    }
}

pub async fn get_client(Path(id): Path<i64>, Extension(_auth_user): Extension<Option<AuthUser>>) -> impl IntoResponse {
    let db = get_connection().await;
    match Client::find_by_id(id).one(db).await {
        Ok(Some(client)) => (StatusCode::OK, ApiResponse::success(client)),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            ApiResponse::<crate::entity::client::Model>::error("Client not found".to_string()),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<crate::entity::client::Model>::error(format!(
                "Failed to get client: {}",
                e
            )),
        ),
    }
}

pub async fn delete_client(
    Path(id): Path<i64>,
    Extension(_auth_user): Extension<Option<AuthUser>>,
) -> impl IntoResponse {
    let db = get_connection().await;
    match Client::delete_by_id(id).exec(db).await {
        Ok(_) => (StatusCode::OK, ApiResponse::success("Client deleted successfully")),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(format!("Failed to delete client: {}", e)),
        ),
    }
}

#[derive(Deserialize)]
pub struct UpdateClientRequest {
    pub name: Option<String>,
    pub upload_limit_gb: Option<f64>,
    pub download_limit_gb: Option<f64>,
    pub traffic_reset_cycle: Option<String>,
    pub is_traffic_exceeded: Option<bool>,
}

pub async fn update_client(
    Path(id): Path<i64>,
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Json(req): Json<UpdateClientRequest>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<crate::entity::client::Model>::error("Not authenticated".to_string())),
    };

    if !auth_user.is_admin {
        return (StatusCode::FORBIDDEN, ApiResponse::<crate::entity::client::Model>::error("Only admin can update client".to_string()));
    }

    let db = get_connection().await;

    let client = match Client::find_by_id(id).one(db).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, ApiResponse::<crate::entity::client::Model>::error("Client not found".to_string())),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<crate::entity::client::Model>::error(format!("Failed to find client: {}", e))),
    };

    let mut client_active: crate::entity::client::ActiveModel = client.into();

    if let Some(name) = req.name {
        client_active.name = Set(name);
    }
    if req.upload_limit_gb.is_some() || req.upload_limit_gb.is_none() {
        client_active.upload_limit_gb = Set(req.upload_limit_gb);
    }
    if req.download_limit_gb.is_some() || req.download_limit_gb.is_none() {
        client_active.download_limit_gb = Set(req.download_limit_gb);
    }
    if let Some(cycle) = req.traffic_reset_cycle {
        client_active.traffic_reset_cycle = Set(cycle);
    }
    if let Some(exceeded) = req.is_traffic_exceeded {
        client_active.is_traffic_exceeded = Set(exceeded);
    }

    client_active.updated_at = Set(Utc::now().naive_utc());

    match client_active.update(db).await {
        Ok(updated) => (StatusCode::OK, ApiResponse::success(updated)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<crate::entity::client::Model>::error(format!("Failed to update client: {}", e))),
    }
}
