use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, PaginatorTrait, QueryFilter, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entity::{Node, node, Client, client},
    migration::get_connection,
    middleware::AuthUser,
    AppState,
};

use super::ApiResponse;

#[derive(Deserialize)]
pub struct CreateNodeRequest {
    pub name: String,
    pub url: String,
    pub secret: Option<String>,
    pub region: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "tunnelAddr")]
    pub tunnel_addr: Option<String>,
    #[serde(rename = "tunnelPort")]
    pub tunnel_port: Option<i32>,
    #[serde(rename = "tunnelProtocol")]
    pub tunnel_protocol: Option<String>,
    #[serde(rename = "kcpConfig")]
    pub kcp_config: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateNodeRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub secret: Option<String>,
    pub region: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "tunnelAddr")]
    pub tunnel_addr: Option<String>,
    #[serde(rename = "tunnelPort")]
    pub tunnel_port: Option<i32>,
    #[serde(rename = "tunnelProtocol")]
    pub tunnel_protocol: Option<String>,
    #[serde(rename = "kcpConfig")]
    pub kcp_config: Option<String>,
}

/// GET /api/nodes — 列出所有节点
pub async fn list_nodes(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<Vec<node::Model>>::error("Not authenticated".to_string())),
    };

    if !auth_user.is_admin {
        return (StatusCode::FORBIDDEN, ApiResponse::<Vec<node::Model>>::error("Only admin can manage nodes".to_string()));
    }

    let db = get_connection().await;
    match Node::find().all(db).await {
        Ok(nodes) => (StatusCode::OK, ApiResponse::success(nodes)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<Vec<node::Model>>::error(format!("Failed to list nodes: {}", e)),
        ),
    }
}

/// POST /api/nodes — 创建节点
pub async fn create_node(
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
    Json(req): Json<CreateNodeRequest>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<node::Model>::error("Not authenticated".to_string())),
    };

    if !auth_user.is_admin {
        return (StatusCode::FORBIDDEN, ApiResponse::<node::Model>::error("Only admin can manage nodes".to_string()));
    }

    let now = Utc::now().naive_utc();
    let new_node = node::ActiveModel {
        id: NotSet,
        name: Set(req.name),
        url: Set(req.url.clone()),
        secret: Set(req.secret.unwrap_or_else(|| Uuid::new_v4().to_string())),
        is_online: Set(false),
        region: Set(req.region),
        description: Set(req.description),
        tunnel_addr: Set(req.tunnel_addr.unwrap_or_default()),
        tunnel_port: Set(req.tunnel_port.unwrap_or(7000)),
        tunnel_protocol: Set(req.tunnel_protocol.unwrap_or_else(|| "quic".to_string())),
        kcp_config: Set(req.kcp_config),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let db = get_connection().await;
    match new_node.insert(db).await {
        Ok(node_model) => {
            // gRPC 模式下节点会主动连接认证，无需手动添加
            (StatusCode::OK, ApiResponse::success(node_model))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<node::Model>::error(format!("Failed to create node: {}", e)),
        ),
    }
}

/// GET /api/nodes/{id} — 获取节点详情
pub async fn get_node(
    Path(id): Path<i64>,
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<node::Model>::error("Not authenticated".to_string())),
    };

    if !auth_user.is_admin {
        return (StatusCode::FORBIDDEN, ApiResponse::<node::Model>::error("Only admin can manage nodes".to_string()));
    }

    let db = get_connection().await;
    match Node::find_by_id(id).one(db).await {
        Ok(Some(node_model)) => (StatusCode::OK, ApiResponse::success(node_model)),
        Ok(None) => (StatusCode::NOT_FOUND, ApiResponse::<node::Model>::error("Node not found".to_string())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<node::Model>::error(format!("Failed to get node: {}", e)),
        ),
    }
}

/// PUT /api/nodes/{id} — 更新节点
pub async fn update_node(
    Path(id): Path<i64>,
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
    Json(req): Json<UpdateNodeRequest>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<node::Model>::error("Not authenticated".to_string())),
    };

    if !auth_user.is_admin {
        return (StatusCode::FORBIDDEN, ApiResponse::<node::Model>::error("Only admin can manage nodes".to_string()));
    }

    let db = get_connection().await;
    let node_model = match Node::find_by_id(id).one(db).await {
        Ok(Some(n)) => n,
        Ok(None) => return (StatusCode::NOT_FOUND, ApiResponse::<node::Model>::error("Node not found".to_string())),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<node::Model>::error(format!("Failed to find node: {}", e))),
    };

    let mut active: node::ActiveModel = node_model.into();

    let mut url_changed = false;
    let mut new_url = String::new();
    let mut new_secret = String::new();

    if let Some(name) = req.name {
        active.name = Set(name);
    }
    if let Some(url) = req.url {
        new_url = url.clone();
        url_changed = true;
        active.url = Set(url);
    }
    if let Some(secret) = req.secret {
        new_secret = secret.clone();
        active.secret = Set(secret);
    }
    if req.region.is_some() {
        active.region = Set(req.region);
    }
    if req.description.is_some() {
        active.description = Set(req.description);
    }
    if let Some(tunnel_addr) = req.tunnel_addr {
        active.tunnel_addr = Set(tunnel_addr);
    }
    if let Some(tunnel_port) = req.tunnel_port {
        active.tunnel_port = Set(tunnel_port);
    }
    if let Some(tunnel_protocol) = req.tunnel_protocol {
        active.tunnel_protocol = Set(tunnel_protocol);
    }
    if req.kcp_config.is_some() {
        active.kcp_config = Set(req.kcp_config);
    }
    active.updated_at = Set(Utc::now().naive_utc());

    match active.update(db).await {
        Ok(updated) => {
            // gRPC 模式下节点会主动重连，无需手动更新连接
            (StatusCode::OK, ApiResponse::success(updated))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<node::Model>::error(format!("Failed to update node: {}", e))),
    }
}

/// DELETE /api/nodes/{id} — 删除节点（需无关联客户端）
pub async fn delete_node(
    Path(id): Path<i64>,
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<&str>::error("Not authenticated".to_string())),
    };

    if !auth_user.is_admin {
        return (StatusCode::FORBIDDEN, ApiResponse::<&str>::error("Only admin can manage nodes".to_string()));
    }

    let db = get_connection().await;

    // 检查是否有关联客户端
    let client_count = Client::find()
        .filter(client::Column::NodeId.eq(id))
        .count(db)
        .await
        .unwrap_or(0);

    if client_count > 0 {
        return (
            StatusCode::BAD_REQUEST,
            ApiResponse::<&str>::error(format!("无法删除节点：仍有 {} 个客户端关联到此节点", client_count)),
        );
    }

    match Node::delete_by_id(id).exec(db).await {
        Ok(_) => {
            // gRPC 模式下节点断开后会自动清理
            (StatusCode::OK, ApiResponse::success("Node deleted successfully"))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(format!("Failed to delete node: {}", e)),
        ),
    }
}

/// POST /api/nodes/{id}/test — 测试节点连接
pub async fn test_node_connection(
    Path(id): Path<i64>,
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<serde_json::Value>::error("Not authenticated".to_string())),
    };

    if !auth_user.is_admin {
        return (StatusCode::FORBIDDEN, ApiResponse::<serde_json::Value>::error("Only admin can manage nodes".to_string()));
    }

    let db = get_connection().await;
    let node_model = match Node::find_by_id(id).one(db).await {
        Ok(Some(n)) => n,
        Ok(None) => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Node not found".to_string())),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<serde_json::Value>::error(format!("Failed to find node: {}", e))),
    };

    // gRPC 模式下检查节点是否已连接
    let connected_ids = app_state.node_manager.get_loaded_node_ids().await;
    let is_online = connected_ids.contains(&id);

    let result = serde_json::json!({
        "online": is_online,
        "node_name": node_model.name,
    });
    (StatusCode::OK, ApiResponse::success(result))
}

/// GET /api/nodes/{id}/status — 获取节点实时状态
pub async fn get_node_status(
    Path(id): Path<i64>,
    Extension(auth_user_opt): Extension<Option<AuthUser>>,
    Extension(app_state): Extension<AppState>,
) -> impl IntoResponse {
    let auth_user = match auth_user_opt {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, ApiResponse::<serde_json::Value>::error("Not authenticated".to_string())),
    };

    if !auth_user.is_admin {
        return (StatusCode::FORBIDDEN, ApiResponse::<serde_json::Value>::error("Only admin can manage nodes".to_string()));
    }

    // gRPC 模式下检查节点是否已连接
    let connected_ids = app_state.node_manager.get_loaded_node_ids().await;
    if !connected_ids.contains(&id) {
        return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Node not connected via gRPC".to_string()));
    }

    // 通过 ProxyControl 获取状态（会通过 gRPC 流发送命令）
    match app_state.proxy_control.get_server_status().await {
        Ok(status) => {
            let result = serde_json::json!({
                "connected_clients": status.connected_clients,
                "active_proxy_count": status.active_proxy_count,
            });
            (StatusCode::OK, ApiResponse::success(result))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<serde_json::Value>::error(format!("Failed to get node status: {}", e)),
        ),
    }
}
