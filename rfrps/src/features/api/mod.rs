use std::sync::Arc;
use axum::middleware::from_fn;
use axum::{Extension, Router};
use axum::routing::{get, post, put};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;
use crate::AppState;
use crate::config_manager::ConfigManager;
use crate::middleware::auth_middleware;
use crate::server::ProxyServer;

mod handlers;

pub async fn enable_api_feature(proxy_server: Arc<ProxyServer>, config_manager: Arc<ConfigManager>)  {
    // 创建应用状态
    let app_state = AppState {
        proxy_server: proxy_server.clone(),
        config_manager: config_manager.clone(),
    };


    // 启动 Web 服务器
    tokio::spawn(async move {
        // 构建 Web 应用
        let api_routes = Router::new()
            // 公开路由（无需认证）
            .route("/auth/login", post(handlers::login))
            // 认证路由（需要登录）
            .route("/auth/me", get(handlers::me))
            // 仪表板路由
            .route("/dashboard/stats/{user_id}", get(handlers::get_user_dashboard_stats))
            .route("/clients", get(handlers::list_clients).post(handlers::create_client))
            .route("/clients/{id}", get(handlers::get_client).delete(handlers::delete_client))
            .route("/clients/{id}/logs", get(handlers::get_client_logs))
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
            // 管理员路由（需要管理员权限）
            .route("/users", get(handlers::list_users).post(handlers::create_user))
            .route("/users/{id}", put(handlers::update_user).delete(handlers::delete_user))
            .route("/users/{id}/clients", get(handlers::get_user_clients))
            .route("/users/{id}/clients/{client_id}", post(handlers::assign_client_to_user).delete(handlers::remove_client_from_user))
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

        let web_addr = String::from("0.0.0.0:3000");
        match tokio::net::TcpListener::bind(web_addr.clone()).await {
            Ok(listener) => {
                match axum::serve(listener, app).await {
                    Ok(_) => {
                        info!("Web服务启动成功！");
                        info!("🌐 Web管理界面: http://{}", web_addr);
                    }
                    Err(err) => {
                        tracing::error!("Web服务启动失败：err => {}", err);
                    }
                }
            }
            Err(err) => {
                tracing::error!("Web服务启动失败：err => {}", err);
            }
        }
    });

}