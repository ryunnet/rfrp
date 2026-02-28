mod config;
mod entity;
mod migration;
mod auth;
mod jwt;
mod middleware;
mod traffic;
mod traffic_limiter;
mod port_limiter;
mod subscription_quota;
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
use clap::{Parser, Subcommand};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, Set};
use sea_orm_migration::MigratorTrait;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use chrono::Utc;
use crate::config::get_config;
use common::protocol::control::ProxyControl;
use common::protocol::auth::ClientAuthProvider;

#[derive(Parser)]
#[command(name = "controller", version, about = "RFRP Controller - 反向代理控制器")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 前台运行控制器
    Start,

    /// 停止运行中的守护进程
    Stop {
        /// PID 文件路径
        #[cfg(unix)]
        #[arg(long, default_value = "/var/run/rfrp-controller.pid")]
        pid_file: String,

        /// PID 文件路径
        #[cfg(windows)]
        #[arg(long, default_value = "rfrp-controller.pid")]
        pid_file: String,
    },

    /// 以守护进程模式运行
    Daemon {
        /// PID 文件路径
        #[cfg(unix)]
        #[arg(long, default_value = "/var/run/rfrp-controller.pid")]
        pid_file: String,

        /// 日志目录路径（按天自动分割）
        #[cfg(unix)]
        #[arg(long, default_value = "./logs")]
        log_dir: String,

        /// PID 文件路径
        #[cfg(windows)]
        #[arg(long, default_value = "rfrp-controller.pid")]
        pid_file: String,

        /// 日志目录路径（按天自动分割）
        #[cfg(windows)]
        #[arg(long, default_value = "./logs")]
        log_dir: String,
    },

    /// 更新到最新版本
    Update,
}

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

// ─── Unix 入口 ───────────────────────────────────────────
// 注意：不使用 #[tokio::main]，因为 daemon 模式需要在 fork 之后才创建 tokio runtime。
// 在 fork 之前创建的 runtime（epoll fd、worker 线程）会在 fork 后损坏，导致网络连接失败。

#[cfg(not(windows))]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Start => {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(run_controller(None))?;
        }

        Command::Stop { pid_file } => {
            stop_daemon_unix(&pid_file)?;
        }

        Command::Daemon {
            pid_file,
            log_dir,
        } => {
            use daemonize::Daemonize;

            // 确保日志目录存在
            fs::create_dir_all(&log_dir).expect("无法创建日志目录");

            println!("启动守护进程模式...");
            println!("PID 文件: {}", pid_file);
            println!("日志目录: {}", log_dir);

            // daemon 模式下 stdout/stderr 重定向到日志目录中的固定文件
            let stdout = std::fs::File::create(format!("{}/daemon.log", log_dir)).expect("无法创建日志文件");
            let stderr = std::fs::File::create(format!("{}/daemon.err", log_dir))
                .expect("无法创建错误日志文件");

            let daemonize = Daemonize::new()
                .pid_file(&pid_file)
                .working_directory(".")
                .stdout(stdout)
                .stderr(stderr);

            match daemonize.start() {
                Ok(_) => println!("守护进程已启动"),
                Err(e) => {
                    eprintln!("启动守护进程失败: {}", e);
                    std::process::exit(1);
                }
            }

            // fork 完成后再创建 tokio runtime，确保 epoll fd 和线程池状态正确
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(run_controller(Some(log_dir)))?;
        }

        Command::Update => {
            update_binary()?;
        }
    }

    Ok(())
}

#[cfg(unix)]
fn stop_daemon_unix(pid_file: &str) -> Result<()> {
    let pid_str = fs::read_to_string(pid_file)
        .map_err(|e| anyhow::anyhow!("无法读取 PID 文件 {}: {}", pid_file, e))?;
    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|e| anyhow::anyhow!("PID 文件内容无效: {}", e))?;

    let ret = unsafe { libc::kill(pid, libc::SIGTERM) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ESRCH) {
            println!("进程 (PID: {}) 已不存在", pid);
        } else {
            return Err(anyhow::anyhow!("停止进程失败 (PID: {}): {}", pid, err));
        }
    } else {
        println!("已发送停止信号到守护进程 (PID: {})", pid);
    }

    fs::remove_file(pid_file).ok();
    Ok(())
}

// ─── Windows 入口 ────────────────────────────────────────

#[cfg(windows)]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Start => {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async { run_controller(None).await })
        }

        Command::Stop { pid_file } => stop_daemon_windows(&pid_file),

        Command::Daemon {
            pid_file,
            log_dir,
        } => start_daemon_windows(&pid_file, &log_dir),

        Command::Update => update_binary(),
    }
}

#[cfg(windows)]
fn start_daemon_windows(pid_file: &str, log_dir: &str) -> Result<()> {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x00000008;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    // 确保日志目录存在
    fs::create_dir_all(log_dir)
        .map_err(|e| anyhow::anyhow!("无法创建日志目录 {}: {}", log_dir, e))?;

    let stdout = fs::File::create(format!("{}/daemon.log", log_dir))
        .map_err(|e| anyhow::anyhow!("无法创建日志文件: {}", e))?;
    let stderr = fs::File::create(format!("{}/daemon.err", log_dir))
        .map_err(|e| anyhow::anyhow!("无法创建错误日志文件: {}", e))?;

    let exe = std::env::current_exe()?;
    let child = std::process::Command::new(&exe)
        .args(["start"])
        .stdout(stdout)
        .stderr(stderr)
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| anyhow::anyhow!("启动守护进程失败: {}", e))?;

    fs::write(pid_file, child.id().to_string())?;

    println!("守护进程已启动 (PID: {})", child.id());
    println!("PID 文件: {}", pid_file);
    println!("日志目录: {}", log_dir);
    println!();
    println!("停止守护进程: controller stop --pid-file {}", pid_file);

    Ok(())
}

#[cfg(windows)]
fn stop_daemon_windows(pid_file: &str) -> Result<()> {
    let pid_str = fs::read_to_string(pid_file)
        .map_err(|e| anyhow::anyhow!("无法读取 PID 文件 {}: {}", pid_file, e))?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|e| anyhow::anyhow!("PID 文件内容无效: {}", e))?;

    unsafe {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, TerminateProcess, PROCESS_TERMINATE,
        };

        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(87) {
                println!("进程 (PID: {}) 已不存在", pid);
                fs::remove_file(pid_file).ok();
                return Ok(());
            }
            return Err(anyhow::anyhow!("无法打开进程 (PID: {}): {}", pid, err));
        }

        let ret = TerminateProcess(handle, 0);
        CloseHandle(handle);

        if ret == 0 {
            let err = std::io::Error::last_os_error();
            return Err(anyhow::anyhow!("停止进程失败 (PID: {}): {}", pid, err));
        }
    }

    println!("已停止守护进程 (PID: {})", pid);
    fs::remove_file(pid_file).ok();
    Ok(())
}

/// 更新二进制文件到最新版本
fn update_binary() -> Result<()> {
    println!("正在检查更新...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("ryunnet")
        .repo_name("rfrp")
        .bin_name("controller")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;

    match status {
        self_update::Status::UpToDate(version) => {
            println!("✓ 已是最新版本: v{}", version);
        }
        self_update::Status::Updated(version) => {
            println!("✓ 成功更新到版本: v{}", version);
            println!("请重启 controller 服务以使用新版本");
        }
    }

    Ok(())
}

/// 运行控制器主逻辑
async fn run_controller(log_dir: Option<String>) -> Result<()> {
    // 安装 rustls CryptoProvider（TLS 需要）
    let _ = rustls::crypto::ring::default_provider().install_default();

    // 初始化 tracing 日志系统
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx::query=warn"));

    // 按天轮转文件日志（daemon 模式）或控制台日志（前台模式）
    if let Some(dir) = &log_dir {
        let file_appender = tracing_appender::rolling::daily(dir, "controller.log");
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_writer(file_appender).with_ansi(false))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer())
            .init();
    }

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
    let _web_handle = api::start_web_server(app_state.clone());

    // 启动 gRPC Server（供 Agent Server 和 Agent Client 连接）
    let _grpc_handle = grpc_server::start_grpc_server(
        config.internal_port,
        node_manager.clone(),
        client_stream_manager.clone(),
        config_manager.clone(),
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
