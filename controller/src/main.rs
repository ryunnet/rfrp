mod config;
mod entity;
mod migration;
mod auth;
mod jwt;
mod middleware;
mod traffic;
mod traffic_limiter;
mod port_limiter;
mod config_manager;
mod api;
mod node_manager;
mod local_auth_provider;
mod client_stream_manager;
mod grpc_agent_server_service;
mod grpc_agent_client_service;
mod grpc_server;
mod geo_ip;

use crate::migration::{get_connection, init_sqlite};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, PaginatorTrait, QueryFilter, Set};
use sea_orm_migration::MigratorTrait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use chrono::Utc;
use crate::config::get_config;
use common::protocol::control::ProxyControl;
use common::protocol::auth::ClientAuthProvider;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub proxy_control: Arc<dyn ProxyControl>,
    pub node_manager: Arc<node_manager::NodeManager>,
    pub auth_provider: Arc<dyn ClientAuthProvider>,
    pub config_manager: Arc<config_manager::ConfigManager>,
    pub client_stream_manager: Arc<client_stream_manager::ClientStreamManager>,
    pub config: Arc<config::Config>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 tracing 日志系统
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx::query=warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    // 读取配置
    let config = get_config().await;
    info!("📋 controller 启动");
    info!("🌐 Web管理端口: {}", config.web_port);
    info!("🔗 内部API端口: {}", config.internal_port);

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

    // 创建多节点管理器
    let node_manager = Arc::new(node_manager::NodeManager::new());
    if let Err(e) = node_manager.load_nodes().await {
        tracing::error!("加载节点失败: {}", e);
    }

    // NodeManager 实现了 ProxyControl trait
    let proxy_control: Arc<dyn ProxyControl> = node_manager.clone();

    // 创建内部认证提供者（controller 直接查询本地 DB）
    let auth_provider: Arc<dyn ClientAuthProvider> = Arc::new(
        local_auth_provider::LocalControllerAuthProvider::new()
    );

    // 创建 Agent Client 流管理器
    let client_stream_manager = Arc::new(client_stream_manager::ClientStreamManager::new());

    let config_arc = Arc::new(config.clone());

    // 创建应用状态
    let app_state = AppState {
        proxy_control: proxy_control.clone(),
        node_manager: node_manager.clone(),
        auth_provider: auth_provider.clone(),
        config_manager: config_manager.clone(),
        client_stream_manager: client_stream_manager.clone(),
        config: config_arc.clone(),
    };

    // 启动 Web API 服务
    let web_handle = api::start_web_server(app_state.clone());

    // 启动 gRPC Server（供 Agent Server 和 Agent Client 连接）
    let grpc_handle = grpc_server::start_grpc_server(
        config.internal_port,
        node_manager.clone(),
        client_stream_manager.clone(),
    );

    // 启动节点健康监控
    start_node_health_monitor(node_manager.clone());

    // 启动客户端健康监控
    start_client_health_monitor(client_stream_manager.clone());

    // 等待终止信号
    info!("✅ 所有服务已启动，等待终止信号...");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("收到 Ctrl+C 信号，正在关闭服务...");
        }
        _ = async {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm = signal(SignalKind::terminate()).expect("failed to listen for SIGTERM");
                sigterm.recv().await;
            }
            #[cfg(not(unix))]
            {
                std::future::pending::<()>().await;
            }
        } => {
            info!("收到 SIGTERM 信号，正在关闭服务...");
        }
    }

    Ok(())
}

/// 启动节点健康监控后台任务
fn start_node_health_monitor(node_manager: Arc<node_manager::NodeManager>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            let results = node_manager.check_all_nodes().await;
            let db = get_connection().await;

            for (node_id, is_online) in results {
                if let Ok(Some(node)) = entity::Node::find_by_id(node_id).one(db).await {
                    let was_online = node.is_online;
                    if was_online != is_online {
                        if is_online {
                            info!("节点 #{} ({}) 已上线", node_id, node.name);
                        } else {
                            tracing::warn!("节点 #{} ({}) 已离线", node_id, node.name);
                        }
                    }

                    let mut active: entity::node::ActiveModel = node.into();
                    active.is_online = Set(is_online);
                    active.updated_at = Set(Utc::now().naive_utc());
                    let _ = active.update(db).await;
                }
            }
        }
    });
}

/// 启动客户端健康监控后台任务
fn start_client_health_monitor(client_stream_manager: Arc<client_stream_manager::ClientStreamManager>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            let results = client_stream_manager.check_all_clients().await;
            let db = get_connection().await;

            for (client_id, is_online) in results {
                if let Ok(Some(client)) = entity::Client::find_by_id(client_id).one(db).await {
                    let was_online = client.is_online;
                    if was_online != is_online {
                        if is_online {
                            info!("客户端 #{} ({}) 已上线", client_id, client.name);
                        } else {
                            tracing::warn!("客户端 #{} ({}) 已离线", client_id, client.name);
                        }
                    }

                    let mut active: entity::client::ActiveModel = client.into();
                    active.is_online = Set(is_online);
                    active.updated_at = Set(Utc::now().naive_utc());
                    let _ = active.update(db).await;
                }
            }
        }
    });
}

/// 初始化 admin 超级管理员用户
async fn initialize_admin_user() {
    use crate::entity::{user::ActiveModel as UserActiveModel, User};

    let db = get_connection().await;

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
                traffic_quota_gb: Set(None),
                traffic_reset_cycle: Set("none".to_string()),
                last_reset_at: Set(None),
                is_traffic_exceeded: Set(false),
                max_port_count: Set(None),
                allowed_port_range: Set(None),
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

                    // 将密码保存到 ./data 目录
                    let data_dir = PathBuf::from("./data");
                    if let Err(e) = std::fs::create_dir_all(&data_dir) {
                        tracing::error!("无法创建 data 目录: {}", e);
                    } else {
                        let password_file = data_dir.join("admin_password.txt");
                        let content = format!(
                            "Admin 初始密码\n═══════════════════════════════════════\n用户名: admin\n密码: {}\n═══════════════════════════════════════\n⚠️ 请妥善保管此文件，登录后建议修改密码并删除此文件！\n",
                            password
                        );
                        match std::fs::write(&password_file, &content) {
                            Ok(_) => {
                                info!("📁 密码已保存到: {}", password_file.display());
                            }
                            Err(e) => {
                                tracing::error!("无法保存密码文件: {}", e);
                            }
                        }
                    }
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
