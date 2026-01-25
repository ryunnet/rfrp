mod server;
mod config;
mod handlers;
mod entity;
mod migration;
mod auth;
mod jwt;
mod middleware;
mod traffic;
mod client_logs;
mod traffic_limiter;
mod config_manager;

use crate::migration::init_sqlite;
use crate::middleware::auth_middleware;
use anyhow::Result;
use axum::{
    routing::{get, post, put, Router},
    middleware::from_fn,
    Extension,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, NotSet, QueryFilter, Set};
use sea_orm_migration::MigratorTrait;
use std::path;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use chrono::Utc;

// 应用状态，用于在handlers之间共享ProxyServer实例
#[derive(Clone)]
pub struct AppState {
    pub proxy_server: Arc<server::ProxyServer>,
    pub config_manager: Arc<config_manager::ConfigManager>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 tracing 日志系统
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    rustls::crypto::ring::default_provider().install_default().unwrap();

    // 读取配置 - 从可执行文件所在目录查找
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap_or(&exe_path);

    // 尝试多个可能的配置文件位置
    let config_path = std::iter::once(exe_dir.join("rfrps.toml"))
        .chain(std::iter::once(path::PathBuf::from("rfrps.toml")))
        .chain(std::iter::once(path::PathBuf::from("../rfrps.toml")))
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("找不到配置文件 rfrps.toml"))?;

    let cfg = config::Config::from_file(&config_path)?;

    info!("📋 加载配置文件: {:?}", config_path.display());
    info!("🌐 QUIC监听端口: {}", cfg.bind_port);
    info!("🌐 Web管理端口: 3000");

    // 初始化数据库
    let db = init_sqlite().await;
    // 运行数据库迁移
    migration::Migrator::up(&db, None).await?;
    info!("✅ 数据库初始化完成");

    // 初始化 admin 用户（如果不存在）
    initialize_admin_user().await;

    // 初始化配置管理器
    let config_manager = Arc::new(config_manager::ConfigManager::new());
    if let Err(e) = config_manager.load_from_db().await {
        tracing::error!("加载系统配置失败: {}", e);
    }

    // 初始化流量管理器
    let traffic_manager = std::sync::Arc::new(traffic::TrafficManager::new());

    // 创建 ProxyServer 实例
    let proxy_server = Arc::new(server::ProxyServer::new(traffic_manager.clone(), config_manager.clone()).unwrap());

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

    // 启动 QUIC 代理服务器
    tokio::spawn(async move {
        // 重置所有客户端为离线状态（服务端重启后清理僵尸状态）
        let db = init_sqlite().await;
        if let Err(e) = reset_all_clients_offline(db).await {
            tracing::warn!("重置客户端状态失败: {}", e);
        }

        let bind_addr = format!("0.0.0.0:{}", cfg.bind_port);
        proxy_server.run(bind_addr).await.unwrap();
    });

    tokio::signal::ctrl_c().await?;
    Ok(())
}

/// 初始化 admin 超级管理员用户
async fn initialize_admin_user() {
    use crate::entity::{user::ActiveModel as UserActiveModel, User};

    let db = migration::get_connection().await;

    // 检查 admin 用户是否已存在
    match User::find()
        .filter(crate::entity::user::Column::Username.eq("admin"))
        .one(db)
        .await
    {
        Ok(Some(_)) => {
            info!("🔐 Admin 用户已存在");
        }
        Ok(None) => {
            // 生成随机密码
            let password = auth::generate_random_password(16);
            let password_hash = match auth::hash_password(&password) {
                Ok(hash) => hash,
                Err(e) => {
                    tracing::error!("Failed to hash admin password: {}", e);
                    return;
                }
            };

            let now = Utc::now().naive_utc();
            let admin_user = UserActiveModel {
                id: NotSet,
                username: Set("admin".to_string()),
                password_hash: Set(password_hash),
                is_admin: Set(true),
                total_bytes_sent: Set(0),
                total_bytes_received: Set(0),
                upload_limit_gb: Set(None),
                download_limit_gb: Set(None),
                traffic_reset_cycle: Set("none".to_string()),
                last_reset_at: Set(None),
                is_traffic_exceeded: Set(false),
                created_at: Set(now),
                updated_at: Set(now),
            };

            match admin_user.insert(db).await {
                Ok(_) => {
                    info!("🔐 Admin 用户已创建");
                    info!("═══════════════════════════════════════════════════════════════");
                    info!("👤 Admin 用户名: admin");
                    info!("🔑 Admin 密码: {}", password);
                    info!("⚠️  请妥善保存此密码，仅在创建时显示一次！");
                    info!("═══════════════════════════════════════════════════════════════");
                }
                Err(e) => {
                    tracing::error!("Failed to create admin user: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to check admin user: {}", e);
        }
    }
}

/// 重置所有客户端为离线状态
async fn reset_all_clients_offline(db: DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    use crate::entity::{Client, client};

    // 查询所有在线的客户端
    let online_clients = Client::find()
        .filter(client::Column::IsOnline.eq(true))
        .all(&db)
        .await?;

    if !online_clients.is_empty() {
        info!("🔄 服务端重启，重置 {} 个客户端状态为离线", online_clients.len());
        for client in online_clients {
            let mut client_active: client::ActiveModel = client.into();
            client_active.is_online = Set(false);
            if let Err(e) = client_active.update(&db).await {
                tracing::error!("Failed to reset client status: {}", e);
            }
        }
    }

    Ok(())
}
