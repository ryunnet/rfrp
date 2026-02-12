use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, Set};
use serde::Deserialize;
use tracing::info;

use crate::server::{entity::Proxy, migration::get_connection, middleware::AuthUser, AppState};

use super::ApiResponse;

#[derive(Deserialize)]
pub struct CreateProxyRequest {
    pub client_id: String,  // 改为 String 以兼容前端
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    #[serde(rename = "localIP")]
    pub local_ip: String,
    #[serde(rename = "localPort")]
    pub local_port: u16,
    #[serde(rename = "remotePort")]
    pub remote_port: u16,
}

#[derive(Deserialize)]
pub struct UpdateProxyRequest {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub proxy_type: Option<String>,
    #[serde(rename = "localIP")]
    pub local_ip: Option<String>,
    #[serde(rename = "localPort")]
    pub local_port: Option<u16>,
    #[serde(rename = "remotePort")]
    pub remote_port: Option<u16>,
    pub enabled: Option<bool>,
}

pub async fn list_proxies(Extension(auth_user_opt): Extension<Option<AuthUser>>) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<Vec<crate::server::entity::proxy::Model>>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    let proxies = if auth_user.is_admin {
        // Admin can see all proxies
        match Proxy::find().all(db).await {
            Ok(proxies) => proxies,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<Vec<crate::server::entity::proxy::Model>>::error(format!(
                        "Failed to list proxies: {}",
                        e
                    )),
                )
            }
        }
    } else {
        // Regular users can only see proxies for their assigned clients
        // First get the user's assigned client IDs
        let user_client_ids = match crate::server::entity::UserClient::find()
            .filter(crate::server::entity::user_client::Column::UserId.eq(auth_user.id))
            .all(db)
            .await
        {
            Ok(user_clients) => user_clients.into_iter().map(|uc| uc.client_id.to_string()).collect::<Vec<_>>(),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<Vec<crate::server::entity::proxy::Model>>::error(format!(
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
            // Get proxies for those clients
            match Proxy::find()
                .filter(crate::server::entity::proxy::Column::ClientId.is_in(user_client_ids))
                .all(db)
                .await
            {
                Ok(proxies) => proxies,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ApiResponse::<Vec<crate::server::entity::proxy::Model>>::error(format!(
                            "Failed to list proxies: {}",
                            e
                        )),
                    )
                }
            }
        }
    };

    (StatusCode::OK, ApiResponse::success(proxies))
}

pub async fn list_proxies_by_client(
    Path(client_id): Path<i64>,
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<Vec<crate::server::entity::proxy::Model>>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    // Check if user has access to this client
    if !auth_user.is_admin {
        match crate::server::entity::UserClient::find()
            .filter(crate::server::entity::user_client::Column::UserId.eq(auth_user.id))
            .filter(crate::server::entity::user_client::Column::ClientId.eq(client_id))
            .one(db)
            .await
        {
            Ok(Some(_)) => {}
            Ok(None) => {
                return (
                    StatusCode::FORBIDDEN,
                    ApiResponse::<Vec<crate::server::entity::proxy::Model>>::error(
                        "Access denied to this client".to_string(),
                    ),
                )
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<Vec<crate::server::entity::proxy::Model>>::error(format!(
                        "Failed to check access: {}",
                        e
                    )),
                )
            }
        }
    }

    match Proxy::find()
        .filter(crate::server::entity::proxy::Column::ClientId.eq(client_id.to_string()))
        .all(db)
        .await
    {
        Ok(proxies) => (StatusCode::OK, ApiResponse::success(proxies)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<Vec<crate::server::entity::proxy::Model>>::error(format!(
                "Failed to list proxies: {}",
                e
            )),
        ),
    }
}

pub async fn create_proxy(
    Extension(_auth_user): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
    Json(req): Json<CreateProxyRequest>,
) -> impl IntoResponse {
    let now = chrono::Utc::now().naive_utc();

    let new_proxy = crate::server::entity::proxy::ActiveModel {
        id: NotSet,
        client_id: Set(req.client_id.clone()),
        name: Set(req.name),
        proxy_type: Set(req.proxy_type),
        local_ip: Set(req.local_ip),
        local_port: Set(req.local_port),
        remote_port: Set(req.remote_port),
        enabled: Set(true),
        total_bytes_sent: Set(0),
        total_bytes_received: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let db = get_connection().await;
    match new_proxy.insert(db).await {
        Ok(proxy) => {
            info!("代理已创建: {} (ID: {}, 客户端: {})", proxy.name, proxy.id, proxy.client_id);

            // 通过 ProxyControl trait 动态启动代理监听器
            let proxy_control = app_state.proxy_control.clone();
            let proxy_id = proxy.id;
            let proxy_name = proxy.name.clone();
            let client_id = req.client_id.clone();

            tokio::spawn(async move {
                if let Err(e) = proxy_control.start_proxy(&client_id, proxy_id).await {
                    tracing::error!("启动代理监听器失败: {}", e);
                } else {
                    info!("代理监听器已动态启动: {}", proxy_name);
                }
            });

            (StatusCode::OK, ApiResponse::success(proxy))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<crate::server::entity::proxy::Model>::error(format!(
                "Failed to create proxy: {}",
                e
            )),
        ),
    }
}

pub async fn update_proxy(
    Path(id): Path<i64>,
    Extension(_auth_user): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
    Json(req): Json<UpdateProxyRequest>,
) -> impl IntoResponse {
    let db = get_connection().await;
    match Proxy::find_by_id(id).one(db).await {
        Ok(Some(proxy)) => {
            let old_enabled = proxy.enabled;
            let client_id = proxy.client_id.clone();
            let mut proxy: crate::server::entity::proxy::ActiveModel = proxy.into();

            if let Some(name) = req.name {
                proxy.name = Set(name);
            }
            if let Some(proxy_type) = req.proxy_type {
                proxy.proxy_type = Set(proxy_type);
            }
            if let Some(local_ip) = req.local_ip {
                proxy.local_ip = Set(local_ip);
            }
            if let Some(local_port) = req.local_port {
                proxy.local_port = Set(local_port);
            }
            if let Some(remote_port) = req.remote_port {
                proxy.remote_port = Set(remote_port);
            }

            let enabled_changed = if let Some(enabled) = req.enabled {
                proxy.enabled = Set(enabled);
                old_enabled != enabled
            } else {
                false
            };

            proxy.updated_at = Set(chrono::Utc::now().naive_utc());

            match proxy.update(&*db).await {
                Ok(updated) => {
                    info!("代理已更新: {} (ID: {})", updated.name, updated.id);

                    // 如果启用状态发生变化，通过 ProxyControl trait 动态启动或停止监听器
                    if enabled_changed {
                        let proxy_control = app_state.proxy_control.clone();
                        let proxy_id = updated.id;
                        let proxy_name = updated.name.clone();
                        let is_enabled = updated.enabled;

                        tokio::spawn(async move {
                            if is_enabled {
                                // 启用代理 - 启动监听器
                                if let Err(e) = proxy_control.start_proxy(&client_id, proxy_id).await {
                                    tracing::error!("启动代理监听器失败: {}", e);
                                } else {
                                    info!("代理监听器已启动: {}", proxy_name);
                                }
                            } else {
                                // 禁用代理 - 停止监听器
                                if let Err(e) = proxy_control.stop_proxy(&client_id, proxy_id).await {
                                    tracing::error!("停止代理监听器失败: {}", e);
                                } else {
                                    info!("代理监听器已停止: {}", proxy_name);
                                }
                            }
                        });
                    }

                    (StatusCode::OK, ApiResponse::success(updated))
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<crate::server::entity::proxy::Model>::error(format!(
                        "Failed to update proxy: {}",
                        e
                    )),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            ApiResponse::<crate::server::entity::proxy::Model>::error("Proxy not found".to_string()),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<crate::server::entity::proxy::Model>::error(format!(
                "Failed to get proxy: {}",
                e
            )),
        ),
    }
}

pub async fn delete_proxy(
    Path(id): Path<i64>,
    Extension(_auth_user): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
) -> impl IntoResponse {
    let db = get_connection().await;

    // 先获取代理信息，用于停止监听器
    let proxy = match Proxy::find_by_id(id).one(db).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                ApiResponse::<&str>::error("Proxy not found".to_string()),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::<&str>::error(format!("Failed to get proxy: {}", e)),
            )
        }
    };

    let client_id = proxy.client_id.clone();
    let proxy_name = proxy.name.clone();

    // 删除代理
    match Proxy::delete_by_id(id).exec(db).await {
        Ok(_) => {
            info!("代理已删除: {} (ID: {})", proxy_name, id);

            // 通过 ProxyControl trait 停止代理监听器
            let proxy_control = app_state.proxy_control.clone();
            tokio::spawn(async move {
                if let Err(e) = proxy_control.stop_proxy(&client_id, id).await {
                    tracing::error!("停止代理监听器失败: {}", e);
                } else {
                    info!("代理监听器已停止: {}", proxy_name);
                }
            });

            (StatusCode::OK, ApiResponse::success("Proxy deleted successfully"))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(format!("Failed to delete proxy: {}", e)),
        ),
    }
}
