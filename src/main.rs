mod client;
mod server;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 安装默认的加密提供者
    rustls::crypto::ring::default_provider().install_default().unwrap();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "server" => {
            // server <quic_port> <forward_port>
            // 示例: rfrp server 9000 8080
            if args.len() < 4 {
                println!("错误: server模式需要两个端口参数");
                println!("用法: rfrp server <quic_port> <forward_port>");
                println!("示例: rfrp server 9000 8080");
                println!("  quic_port: QUIC协议监听端口（客户端连接此端口）");
                println!("  forward_port: 转发端口（外部用户访问此端口）");
                return Ok(());
            }

            let quic_port = args[2].parse::<u16>()?;
            let forward_port = args[3].parse::<u16>()?;
            let bind_addr = format!("0.0.0.0:{}", quic_port).parse()?;

            let srv = server::Server::new(bind_addr, forward_port)?;
            srv.run().await?;
        }
        "client" => {
            // client <server_addr> <server_port> <target_addr>
            // 示例: rfrp client 192.168.1.100 9000 10.0.13.1:22
            if args.len() < 5 {
                println!("错误: client模式需要三个参数");
                println!("用法: rfrp client <server_addr> <server_port> <target_addr>");
                println!("示例: rfrp client 192.168.1.100 9000 10.0.13.1:22");
                println!("  server_addr: 服务器地址");
                println!("  server_port: 服务器的QUIC端口");
                println!("  target_addr: 目标服务地址");
                return Ok(());
            }

            let server_addr = args[2].clone();
            let server_port = args[3].parse::<u16>()?;
            let target_addr = args[4].clone();

            let server_socket = format!("{}:{}", server_addr, server_port).parse()?;
            let client = client::Client::new(server_socket, target_addr)?;
            client.run().await?;
        }
        _ => {
            println!("未知命令: {}", args[1]);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("RFRP - 基于QUIC协议的端口转发工具");
    println!();
    println!("用法:");
    println!("  rfrp server <quic_port> <forward_port>");
    println!("      启动服务器端");
    println!("      quic_port: QUIC协议监听端口（客户端连接此端口）");
    println!("      forward_port: 转发端口（外部用户访问此端口）");
    println!();
    println!("  rfrp client <server_addr> <server_port> <target_addr>");
    println!("      启动客户端，连接到服务器");
    println!("      server_addr: 服务器地址");
    println!("      server_port: 服务器的QUIC端口");
    println!("      target_addr: 目标服务地址（格式: IP:PORT）");
    println!();
    println!("示例:");
    println!("  # 在服务器上运行 (假设服务器IP是 192.168.1.100)");
    println!("  # 监听QUIC端口9000，转发端口8080");
    println!("  rfrp server 9000 8080");
    println!();
    println!("  # 在客户端上运行");
    println!("  # 连接到服务器192.168.1.100:9000");
    println!("  # 将访问转发到 10.0.13.1:22");
    println!("  rfrp client 192.168.1.100 9000 10.0.13.1:22");
    println!();
    println!("工作原理:");
    println!("  1. 用户通过SSH连接到服务器的8080端口");
    println!("     ssh -p 8080 user@server_ip");
    println!("  2. 服务器通过QUIC协议将流量转发给客户端");
    println!("  3. 客户端将流量转发到目标服务 10.0.13.1:22");
    println!("  4. 基于QUIC协议，支持多路复用和更好的性能");
}
