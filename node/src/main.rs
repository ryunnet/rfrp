mod server;

use clap::Parser;

#[cfg(unix)]
use daemonize::Daemonize;
#[cfg(unix)]
use std::fs::File;

#[derive(Parser)]
#[command(name = "node", version, about = "RFRP Node - 反向代理节点服务器")]
struct Cli {
    /// Controller gRPC 地址（例如 http://controller:3100）
    #[arg(long)]
    controller_url: String,

    /// 节点密钥
    #[arg(long)]
    token: String,

    /// 隧道监听端口（默认 7000）
    #[arg(long, default_value = "7000")]
    bind_port: u16,

    /// 隧道协议：quic 或 kcp（默认 quic）
    #[arg(long, default_value = "quic")]
    protocol: String,

    /// 以守护进程模式运行（仅 Unix 系统）
    #[cfg(unix)]
    #[arg(long)]
    daemon: bool,

    /// PID 文件路径（守护进程模式）
    #[cfg(unix)]
    #[arg(long, default_value = "/var/run/rfrp-node.pid")]
    pid_file: String,

    /// 日志文件路径（守护进程模式）
    #[cfg(unix)]
    #[arg(long, default_value = "/var/log/rfrp-node.log")]
    log_file: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 安装 rustls 加密提供者（只调用一次）
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

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

    server::run_server_controller_mode(
        cli.controller_url,
        cli.token,
        cli.bind_port,
        cli.protocol,
    ).await?;

    Ok(())
}
