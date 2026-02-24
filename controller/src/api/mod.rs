use axum::middleware::from_fn;
use axum::{Extension, Router};
use axum::routing::{get, post, put};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;
use crate::AppState;
use crate::middleware::auth_middleware;

pub mod handlers;

/// 启动 Web API 服务
pub fn start_web_server(app_state: AppState) -> tokio::task::JoinHandle<()> {
    let web_port = app_state.config.web_port;

    tokio::spawn(async move {
        // 构建 Web 应用
        let api_routes = Router::new()
            // 公开路由（无需认证）
            .route("/auth/login", post(handlers::login))
            .route("/client/connect-config", post(handlers::get_client_connect_config))
            // 认证路由（需要登录）
            .route("/auth/me", get(handlers::me))
            // 仪表板路由
            .route("/dashboard/stats/{user_id}", get(handlers::get_user_dashboard_stats))
            .route("/clients", get(handlers::list_clients).post(handlers::create_client))
            .route("/clients/{id}", get(handlers::get_client).delete(handlers::delete_client))
            .route("/clients/{id}/logs", get(handlers::get_client_logs))
            .route("/clients/{id}/traffic", get(handlers::get_client_traffic))
            .route("/clients/{id}/allocate-quota", post(handlers::allocate_client_quota))
            .route("/proxies", get(handlers::list_proxies).post(handlers::create_proxy))
            .route("/proxies/{id}", put(handlers::update_proxy).delete(handlers::delete_proxy))
            .route("/clients/{id}/proxies", get(handlers::list_proxies_by_client))
            // 流量统计路由
            .route("/traffic/overview", get(handlers::get_traffic_overview_handler))
            .route("/traffic/users/{id}", get(handlers::get_user_traffic_handler))
            // 系统配置路由
            .route("/system/configs", get(handlers::get_configs))
            .route("/system/configs/update", post(handlers::update_config))
            .route("/system/configs/batch", post(handlers::batch_update_configs))
            .route("/system/restart", post(handlers::restart_system))
            // 管理员路由（需要管理员权限）
            .route("/users", get(handlers::list_users).post(handlers::create_user))
            .route("/users/{id}", put(handlers::update_user).delete(handlers::delete_user))
            .route("/users/{id}/nodes", get(handlers::get_user_nodes))
            .route("/users/{id}/nodes/{node_id}", post(handlers::assign_node_to_user).delete(handlers::remove_node_from_user))
            .route("/users/{id}/adjust-quota", post(handlers::adjust_user_quota))
            .route("/users/{id}/quota-info", get(handlers::get_user_quota_info))
            // 节点管理路由（管理员权限）
            .route("/nodes", get(handlers::list_nodes).post(handlers::create_node))
            .route("/nodes/{id}", get(handlers::get_node).put(handlers::update_node).delete(handlers::delete_node))
            .route("/nodes/{id}/test", post(handlers::test_node_connection))
            .route("/nodes/{id}/status", get(handlers::get_node_status))
            // 订阅管理路由
            .route("/subscriptions", get(handlers::list_subscriptions).post(handlers::create_subscription))
            .route("/subscriptions/active", get(handlers::list_active_subscriptions))
            .route("/subscriptions/{id}", get(handlers::get_subscription).put(handlers::update_subscription).delete(handlers::delete_subscription))
            // 用户订阅路由
            .route("/user-subscriptions", get(handlers::list_user_subscriptions).post(handlers::create_user_subscription))
            .route("/user-subscriptions/{id}", put(handlers::update_user_subscription).delete(handlers::delete_user_subscription))
            .route("/users/{user_id}/subscriptions", get(handlers::get_user_subscriptions))
            .route("/users/{user_id}/subscriptions/active", get(handlers::get_user_active_subscription))
            // 应用认证中间件
            .layer(from_fn(auth_middleware))
            // 添加应用状态
            .layer(Extension(app_state));

        let app = Router::new()
            // API 路由
            .nest("/api", api_routes)
            // 静态文件服务，带 SPA fallback
            .fallback_service(
                ServeDir::new("dist")
                    .fallback(ServeFile::new("dist/index.html"))
            )
            .layer(CorsLayer::permissive());

        let web_addr = format!("0.0.0.0:{}", web_port);
        match tokio::net::TcpListener::bind(web_addr.clone()).await {
            Ok(listener) => {
                info!("🌐 Web管理界面: http://{}", web_addr);
                if let Err(err) = axum::serve(listener, app).await {
                    tracing::error!("Web服务错误：{}", err);
                }
            }
            Err(err) => {
                tracing::error!("Web服务启动失败：{}", err);
            }
        }
    })
}
