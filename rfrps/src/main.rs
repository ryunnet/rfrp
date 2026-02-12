mod server;
mod config;
mod entity;
mod migration;
mod auth;
mod jwt;
mod middleware;
mod traffic;
mod client_logs;
mod traffic_limiter;
mod config_manager;
mod features;

use crate::migration::{get_connection, init_sqlite};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, Set};
use sea_orm_migration::MigratorTrait;
use std::path::{self, PathBuf};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use chrono::Utc;
use crate::config::get_config;

// 应用状态，用于在handlers之间共享ProxyServer实例
#[derive(Clone)]
pub struct AppState {
    pub proxy_server: Arc<server::ProxyServer>,
    pub config_manager: Arc<config_manager::ConfigManager>,
    pub config: Arc<config::Config>,
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

    let config = get_config().await;
    info!("📋 加载配置文件: {:?}", config_path.display());
    info!("🌐 QUIC监听端口: {}", config.bind_port);
    info!("🌐 Web管理端口: 3000");

    // 初始化数据库
    let db = init_sqlite().await;
    // 运行数据库迁移
    migration::Migrator::up(&db, None).await?;
    info!("✅ 数据库初始化完成");

    // 初始化 admin 用户（如果不存在）
    initialize_admin_user().await;

    features::init_features().await;

    // 等待终止信号
    info!("✅ 所有服务已启动，等待终止信号...");

    // 使用 select! 来同时监听多个信号
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