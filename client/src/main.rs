mod client;

#[cfg(windows)]
mod windows_service;

use clap::Parser;

#[cfg(unix)]
use daemonize::Daemonize;
#[cfg(unix)]
use std::fs::File;

#[derive(Parser)]
#[command(name = "client", version, about = "RFRP Client - 反向代理客户端")]
struct Cli {
    /// Controller 地址（例如 http://controller:3100）
    #[arg(long)]
    controller_url: Option<String>,

    /// 客户端 Token
    #[arg(long)]
    token: Option<String>,

    /// 以守护进程模式运行（仅 Unix 系统）
    #[cfg(unix)]
    #[arg(long)]
    daemon: bool,

    /// PID 文件路径（守护进程模式）
    #[cfg(unix)]
    #[arg(long, default_value = "/var/run/rfrp-client.pid")]
    pid_file: String,

    /// 日志文件路径（守护进程模式）
    #[cfg(unix)]
    #[arg(long, default_value = "/var/log/rfrp-client.log")]
    log_file: String,

    /// 安装为 Windows 服务（仅 Windows 系统）
    #[cfg(windows)]
    #[arg(long)]
    install_service: bool,

    /// 卸载 Windows 服务（仅 Windows 系统）
    #[cfg(windows)]
    #[arg(long)]
    uninstall_service: bool,

    /// 以 Windows 服务模式运行（由 SCM 调用，用户不应直接使用）
    #[cfg(windows)]
    #[arg(long, hide = true)]
    service: bool,
}

#[cfg(not(windows))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 安装 rustls 加密提供者（只调用一次）
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    let controller_url = cli.controller_url.expect("缺少 --controller-url 参数");
    let token = cli.token.expect("缺少 --token 参数");

    #[cfg(unix)]
    if cli.daemon {
        println!("启动守护进程模式...");
        println!("PID 文件: {}", cli.pid_file);
        println!("日志文件: {}", cli.log_file);

        let stdout = File::create(&cli.log_file)
            .expect("无法创建日志文件");
        let stderr = File::create(format!("{}.err", cli.log_file))
            .expect("无法创建错误日志文件");

        let daemonize = Daemonize::new()
            .pid_file(&cli.pid_file)
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
    }

    client::run_client(controller_url, token).await?;

    Ok(())
}

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    // 安装 rustls 加密提供者（只调用一次）
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    // Windows 服务管理命令
    if cli.install_service {
        let controller_url = cli.controller_url.expect("缺少 --controller-url 参数");
        let token = cli.token.expect("缺少 --token 参数");
        return windows_service::install_service(&controller_url, &token);
    }

    if cli.uninstall_service {
        return windows_service::uninstall_service();
    }

    if cli.service {
        // 由 Windows 服务控制管理器调用
        return windows_service::run_service();
    }

    // 普通前台运行模式
    let controller_url = cli.controller_url.expect("缺少 --controller-url 参数");
    let token = cli.token.expect("缺少 --token 参数");

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        client::run_client(controller_url, token).await
    })
}
