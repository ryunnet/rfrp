mod client;

use clap::Parser;

#[derive(Parser)]
#[command(name = "client", version, about = "RFRP Client - 反向代理客户端")]
struct Cli {
    /// Controller 地址（例如 http://controller:3100）
    #[arg(long)]
    controller_url: String,

    /// 客户端 Token
    #[arg(long)]
    token: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 安装 rustls 加密提供者（只调用一次）
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    client::run_client(cli.controller_url, cli.token).await?;

    Ok(())
}
