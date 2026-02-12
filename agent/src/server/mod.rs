pub mod proxy_server;
pub mod config;
pub mod entity;
pub mod migration;
pub mod auth;
pub mod jwt;
pub mod middleware;
pub mod traffic;
pub mod client_logs;
pub mod traffic_limiter;
pub mod config_manager;
pub mod features;
pub mod local_proxy_control;
pub mod local_auth_provider;
pub mod remote_auth_provider;
pub mod internal_api;

use crate::server::migration::{get_connection, init_sqlite};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, Set};
use sea_orm_migration::MigratorTrait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use chrono::Utc;
use crate::server::config::get_config;
use common::protocol::control::ProxyControl;
use common::protocol::auth::ClientAuthProvider;
use common::protocol::node_register::{NodeRegisterRequest, NodeRegisterResponse};

// 应用状态，用于在handlers之间共享ProxyServer实例
#[derive(Clone)]
pub struct AppState {
    pub proxy_server: Arc<proxy_server::ProxyServer>,
    pub proxy_control: Arc<dyn ProxyControl>,
    pub auth_provider: Arc<dyn ClientAuthProvider>,
    pub config_manager: Arc<config_manager::ConfigManager>,
    pub config: Arc<config::Config>,
}

pub async fn run_server(config_path: String) -> Result<()> {
    // 初始化 tracing 日志系统
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    // 初始化配置路径
    config::init_config_path(config_path.clone()).await;

    // 读取配置
    let config = get_config().await;
    info!("加载配置文件: {}", config_path);
    info!("QUIC监听端口: {}", config.bind_port);
    info!("Web管理端口: 3000");

    // 初始化数据库
    let db = init_sqlite().await;
    // 运行数据库迁移
    migration::Migrator::up(&db, None).await?;
    info!("数据库初始化完成");

    // 初始化 admin 用户（如果不存在）
    initialize_admin_user().await;

    features::init_features().await;

    // 等待终止信号
    info!("所有服务已启动，等待终止信号...");

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

/// Controller 模式启动
///
/// 不使用本地数据库、JWT、Web UI，所有管理由 Controller 统一负责。
/// 仅运行隧道服务和内部 API。
pub async fn run_server_controller_mode(
    controller_url: String,
    token: String,
    bind_port: u16,
    internal_port: u16,
    protocol: String,
) -> Result<()> {
    // 初始化 tracing 日志系统
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    info!("Agent Server 启动 (Controller 模式)");
    info!("Controller: {}", controller_url);
    info!("隧道端口: {}", bind_port);
    info!("内部API端口: {}", internal_port);
    info!("隧道协议: {}", protocol);

    // 向 Controller 注册（带重试）
    let register_response = register_to_controller(
        &controller_url,
        &token,
        bind_port,
        internal_port,
        &protocol,
    ).await?;

    info!("注册成功: 节点 #{} ({})", register_response.node_id, register_response.node_name);

    let controller_internal_url = register_response.controller_internal_url.clone();
    let internal_secret = register_response.internal_secret.clone();

    // 创建远程认证提供者
    let auth_provider: Arc<dyn ClientAuthProvider> = Arc::new(
        remote_auth_provider::RemoteClientAuthProvider::new(
            controller_internal_url.clone(),
            internal_secret.clone(),
        )
    );

    // 创建远程流量管理器
    let traffic_manager = Arc::new(
        traffic::TrafficManager::new_remote(
            controller_internal_url.clone(),
            internal_secret.clone(),
        )
    );

    // 创建配置管理器（使用默认值，不加载 DB）
    let config_manager = Arc::new(config_manager::ConfigManager::new());

    // 创建 ProxyServer
    let proxy_server = Arc::new(
        proxy_server::ProxyServer::new(
            traffic_manager.clone(),
            config_manager.clone(),
            auth_provider.clone(),
        )?
    );

    // 创建本地代理控制实例
    let proxy_control: Arc<dyn ProxyControl> = Arc::new(local_proxy_control::LocalProxyControl::new(
        proxy_server.get_listener_manager(),
        proxy_server.get_client_connections(),
        proxy_server.get_tunnel_connections(),
    ));

    // 创建 ConnectionProvider
    let conn_provider = proxy_server::ConnectionProvider::new(
        proxy_server.get_client_connections(),
        proxy_server.get_tunnel_connections(),
    );

    // 启动内部 API（供 controller 调用）
    internal_api::start_agent_internal_api(
        internal_port,
        internal_secret,
        proxy_control.clone(),
        auth_provider.clone(),
        proxy_server.get_listener_manager(),
        conn_provider,
    );

    // 启动隧道服务
    let bind_addr = format!("0.0.0.0:{}", bind_port);
    let proxy_server_clone = proxy_server.clone();

    match protocol.as_str() {
        "kcp" => {
            info!("启动 KCP 隧道服务: {}", bind_addr);
            tokio::spawn(async move {
                if let Err(e) = proxy_server_clone.run_kcp(bind_addr, None).await {
                    error!("KCP server error: {}", e);
                }
            });
        }
        _ => {
            info!("启动 QUIC 隧道服务: {}", bind_addr);
            tokio::spawn(async move {
                if let Err(e) = proxy_server_clone.run(bind_addr).await {
                    error!("QUIC server error: {}", e);
                }
            });
        }
    }

    // 等待终止信号
    info!("所有服务已启动，等待终止信号...");

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

/// 向 Controller 注册节点（带重试）
async fn register_to_controller(
    controller_url: &str,
    token: &str,
    tunnel_port: u16,
    internal_port: u16,
    tunnel_protocol: &str,
) -> Result<NodeRegisterResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}/internal/nodes/register", controller_url);
    let req = NodeRegisterRequest {
        token: token.to_string(),
        tunnel_port,
        internal_port,
        tunnel_protocol: tunnel_protocol.to_string(),
    };

    let mut retry_count = 0u32;
    loop {
        match client.post(&url).json(&req).send().await {
            Ok(resp) if resp.status().is_success() => {
                let response: NodeRegisterResponse = resp.json().await?;
                return Ok(response);
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                error!("注册失败 (HTTP {}): {}", status, body);
            }
            Err(e) => {
                error!("注册请求失败: {}", e);
            }
        }

        retry_count += 1;
        let delay = Duration::from_secs(std::cmp::min(5 * retry_count as u64, 30));
        info!("{}秒后重试注册 (第{}次)...", delay.as_secs(), retry_count);
        tokio::time::sleep(delay).await;
    }
}

/// 初始化 admin 超级管理员用户
async fn initialize_admin_user() {
    use crate::server::entity::{user::ActiveModel as UserActiveModel, User};

    let db = get_connection().await;

    // 检查 admin 用户是否已存在
    match User::find()
        .filter(crate::server::entity::user::Column::Username.eq("admin"))
        .one(db)
        .await
    {
        Ok(Some(_)) => {
            info!("Admin 用户已存在");
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
                    info!("Admin 用户已创建");
                    info!("═══════════════════════════════════════════════════════════════");
                    info!("Admin 用户名: admin");
                    info!("Admin 密码: {}", password);
                    info!("请妥善保存此密码，仅在创建时显示一次！");
                    info!("═══════════════════════════════════════════════════════════════");

                    // 将密码保存到 ./data 目录
                    let data_dir = PathBuf::from("./data");
                    if let Err(e) = std::fs::create_dir_all(&data_dir) {
                        tracing::error!("无法创建 data 目录: {}", e);
                    } else {
                        let password_file = data_dir.join("admin_password.txt");
                        let content = format!(
                            "Admin 初始密码\n═══════════════════════════════════════\n用户名: admin\n密码: {}\n═══════════════════════════════════════\n请妥善保管此文件，登录后建议修改密码并删除此文件！\n",
                            password
                        );
                        match std::fs::write(&password_file, &content) {
                            Ok(_) => {
                                info!("密码已保存到: {}", password_file.display());
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
