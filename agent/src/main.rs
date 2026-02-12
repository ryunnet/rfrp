mod server;
mod client;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent", version, about = "RFRP Agent - 反向代理服务端与客户端")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 以服务端模式运行
    Server {
        /// 配置文件路径（独立模式）
        #[arg(short, long)]
        config: Option<String>,

        /// Controller 内部 API 地址（controller 模式，例如 http://controller:3100）
        #[arg(long)]
        controller_url: Option<String>,

        /// 节点密钥（controller 模式必需）
        #[arg(long)]
        token: Option<String>,

        /// 隧道监听端口（默认 7000）
        #[arg(long, default_value = "7000")]
        bind_port: u16,

        /// 内部 API 端口（默认 7001）
        #[arg(long, default_value = "7001")]
        internal_port: u16,

        /// 隧道协议：quic 或 kcp（默认 quic）
        #[arg(long, default_value = "quic")]
        protocol: String,
    },
    /// 以客户端模式运行
    Client {
        /// 配置文件路径（与 --controller-url 互斥）
        #[arg(short, long)]
        config: Option<String>,

        /// Controller 地址（例如 http://controller:3000）
        #[arg(long)]
        controller_url: Option<String>,

        /// 客户端 Token（controller 模式必需）
        #[arg(long)]
        token: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 安装 rustls 加密提供者（只调用一次）
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    match cli.command {
        Commands::Server { config, controller_url, token, bind_port, internal_port, protocol } => {
            if let Some(controller_url) = controller_url {
                // Controller 模式
                let token = token.expect("--token is required in controller mode");
                server::run_server_controller_mode(
                    controller_url,
                    token,
                    bind_port,
                    internal_port,
                    protocol,
                ).await?;
            } else {
                // 独立模式
                let config_path = config.unwrap_or_else(|| "rfrps.toml".to_string());
                server::run_server(config_path).await?;
            }
        }
        Commands::Client { config, controller_url, token } => {
            client::run_client(config, controller_url, token).await?;
        }
    }

    Ok(())
}
