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
    /// 以服务端模式运行（需要 Controller）
    Server {
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
    },
    /// 以客户端模式运行
    Client {
        /// Controller 地址（例如 http://controller:3100）
        #[arg(long)]
        controller_url: String,

        /// 客户端 Token
        #[arg(long)]
        token: String,
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
        Commands::Server { controller_url, token, bind_port, protocol } => {
            server::run_server_controller_mode(
                controller_url,
                token,
                bind_port,
                protocol,
            ).await?;
        }
        Commands::Client { controller_url, token } => {
            client::run_client(controller_url, token).await?;
        }
    }

    Ok(())
}
