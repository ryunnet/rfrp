use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, Set};
use serde::Deserialize;
use tracing::info;

use crate::{entity::Proxy, migration::get_connection, middleware::AuthUser, AppState};

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
    #[serde(rename = "nodeId")]
    pub node_id: Option<i64>,
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
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<Vec<crate::entity::proxy::Model>>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    let proxies = if auth_user.is_admin {
        // Admin can see all proxies
        match Proxy::find().all(db).await {
            Ok(proxies) => proxies,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<Vec<crate::entity::proxy::Model>>::error(format!(
                        "Failed to list proxies: {}",
                        e
                    )),
                )
            }
        }
    } else {
        // Regular users can only see proxies for their own clients
        let client_ids = match crate::entity::Client::find()
            .filter(crate::entity::client::Column::UserId.eq(auth_user.id))
            .all(db)
            .await
        {
            Ok(clients) => clients.into_iter().map(|c| c.id.to_string()).collect::<Vec<_>>(),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<Vec<crate::entity::proxy::Model>>::error(format!(
                        "Failed to get clients: {}",
                        e
                    )),
                )
            }
        };

        if client_ids.is_empty() {
            vec![]
        } else {
            // Get proxies for those clients
            match Proxy::find()
                .filter(crate::entity::proxy::Column::ClientId.is_in(client_ids))
                .all(db)
                .await
            {
                Ok(proxies) => proxies,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ApiResponse::<Vec<crate::entity::proxy::Model>>::error(format!(
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
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<Vec<crate::entity::proxy::Model>>::error("Not authenticated".to_string())),
    };
    let db = get_connection().await;

    // Check if user has access to this client (via node binding)
    if !auth_user.is_admin {
        // First get the client's node_id
        let client = match crate::entity::Client::find_by_id(client_id).one(db).await {
            Ok(Some(c)) => c,
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    ApiResponse::<Vec<crate::entity::proxy::Model>>::error(
                        "Client not found".to_string(),
                    ),
                )
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<Vec<crate::entity::proxy::Model>>::error(format!(
                        "Failed to check access: {}",
                        e
                    )),
                )
            }
        };

        // Check if user owns the client
        if client.user_id != Some(auth_user.id) {
            return (
                StatusCode::FORBIDDEN,
                ApiResponse::<Vec<crate::entity::proxy::Model>>::error(
                    "Access denied to this client".to_string(),
                ),
            )
        }
    }

    match Proxy::find()
        .filter(crate::entity::proxy::Column::ClientId.eq(client_id.to_string()))
        .all(db)
        .await
    {
        Ok(proxies) => (StatusCode::OK, ApiResponse::success(proxies)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<Vec<crate::entity::proxy::Model>>::error(format!(
                "Failed to list proxies: {}",
                e
            )),
        ),
    }
}

pub async fn create_proxy(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
    Json(req): Json<CreateProxyRequest>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<crate::entity::proxy::Model>::error("未认证".to_string())),
    };

    let db = get_connection().await;

    // 验证节点权限
    if let Some(node_id) = req.node_id {
        // 获取节点信息
        let node = match crate::entity::Node::find_by_id(node_id).one(db).await {
            Ok(Some(n)) => n,
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    ApiResponse::<crate::entity::proxy::Model>::error("节点不存在".to_string()),
                )
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<crate::entity::proxy::Model>::error(format!("查询节点失败: {}", e)),
                )
            }
        };

        // 如果是独享节点，需要检查用户是否有权限
        if node.node_type == "dedicated" && !auth_user.is_admin {
            // 获取客户端所属用户
            let client = match crate::entity::Client::find()
                .filter(crate::entity::client::Column::Id.eq(req.client_id.parse::<i64>().unwrap_or(0)))
                .one(db)
                .await
            {
                Ok(Some(c)) => c,
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        ApiResponse::<crate::entity::proxy::Model>::error("客户端不存在".to_string()),
                    )
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ApiResponse::<crate::entity::proxy::Model>::error(format!("查询客户端失败: {}", e)),
                    )
                }
            };

            // 检查客户端是否属于当前用户
            if client.user_id != Some(auth_user.id) {
                return (
                    StatusCode::FORBIDDEN,
                    ApiResponse::<crate::entity::proxy::Model>::error("无权访问此客户端".to_string()),
                );
            }

            // 检查节点是否分配给了该用户
            let user_node = crate::entity::UserNode::find()
                .filter(crate::entity::user_node::Column::UserId.eq(auth_user.id))
                .filter(crate::entity::user_node::Column::NodeId.eq(node_id))
                .one(db)
                .await;

            match user_node {
                Ok(Some(_)) => {
                    // 用户有权限使用此独享节点
                }
                Ok(None) => {
                    return (
                        StatusCode::FORBIDDEN,
                        ApiResponse::<crate::entity::proxy::Model>::error("此独享节点未分配给您，无法使用".to_string()),
                    );
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ApiResponse::<crate::entity::proxy::Model>::error(format!("检查节点权限失败: {}", e)),
                    );
                }
            }
        }
        // 共享节点对所有用户可用，无需额外检查
    }

    let now = chrono::Utc::now().naive_utc();

    let new_proxy = crate::entity::proxy::ActiveModel {
        id: NotSet,
        client_id: Set(req.client_id.clone()),
        name: Set(req.name),
        proxy_type: Set(req.proxy_type),
        local_ip: Set(req.local_ip),
        local_port: Set(req.local_port),
        remote_port: Set(req.remote_port),
        enabled: Set(true),
        node_id: Set(req.node_id),
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

            // 通知 Agent Client 代理配置已变更
            let csm = app_state.client_stream_manager.clone();
            let client_id_notify = req.client_id.clone();
            tokio::spawn(async move {
                csm.notify_proxy_change(&client_id_notify).await;
            });

            (StatusCode::OK, ApiResponse::success(proxy))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<crate::entity::proxy::Model>::error(format!(
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
            let old_proxy_type = proxy.proxy_type.clone();
            let old_local_ip = proxy.local_ip.clone();
            let old_local_port = proxy.local_port;
            let old_remote_port = proxy.remote_port;
            let client_id = proxy.client_id.clone();
            let mut proxy: crate::entity::proxy::ActiveModel = proxy.into();

            let mut config_changed = false;

            if let Some(name) = req.name {
                proxy.name = Set(name);
            }
            if let Some(proxy_type) = req.proxy_type {
                if proxy_type != old_proxy_type {
                    config_changed = true;
                }
                proxy.proxy_type = Set(proxy_type);
            }
            if let Some(local_ip) = req.local_ip {
                if local_ip != old_local_ip {
                    config_changed = true;
                }
                proxy.local_ip = Set(local_ip);
            }
            if let Some(local_port) = req.local_port {
                if local_port != old_local_port {
                    config_changed = true;
                }
                proxy.local_port = Set(local_port);
            }
            if let Some(remote_port) = req.remote_port {
                if remote_port != old_remote_port {
                    config_changed = true;
                }
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

                    let need_restart = enabled_changed || (config_changed && updated.enabled);

                    if need_restart {
                        let proxy_control = app_state.proxy_control.clone();
                        let proxy_id = updated.id;
                        let proxy_name = updated.name.clone();
                        let is_enabled = updated.enabled;
                        let client_id_clone = client_id.clone();

                        tokio::spawn(async move {
                            // 先停止旧监听器
                            if let Err(e) = proxy_control.stop_proxy(&client_id_clone, proxy_id).await {
                                tracing::warn!("停止旧代理监听器: {}", e);
                            }

                            if is_enabled {
                                // 启动新监听器
                                if let Err(e) = proxy_control.start_proxy(&client_id_clone, proxy_id).await {
                                    tracing::error!("启动代理监听器失败: {}", e);
                                } else {
                                    info!("代理监听器已重启: {}", proxy_name);
                                }
                            } else {
                                info!("代理监听器已停止: {}", proxy_name);
                            }
                        });
                    }

                    // 通知 Agent Client 代理配置已变更
                    if enabled_changed || config_changed {
                        let csm = app_state.client_stream_manager.clone();
                        let client_id_notify = client_id.clone();
                        tokio::spawn(async move {
                            csm.notify_proxy_change(&client_id_notify).await;
                        });
                    }

                    (StatusCode::OK, ApiResponse::success(updated))
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<crate::entity::proxy::Model>::error(format!(
                        "Failed to update proxy: {}",
                        e
                    )),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            ApiResponse::<crate::entity::proxy::Model>::error("Proxy not found".to_string()),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<crate::entity::proxy::Model>::error(format!(
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

            // 通知 Agent Client 代理配置已变更
            let csm = app_state.client_stream_manager.clone();
            let client_id_notify = proxy.client_id.clone();
            tokio::spawn(async move {
                csm.notify_proxy_change(&client_id_notify).await;
            });

            (StatusCode::OK, ApiResponse::success("Proxy deleted successfully"))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(format!("Failed to delete proxy: {}", e)),
        ),
    }
}
