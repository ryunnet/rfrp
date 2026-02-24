pub mod proxy_server;
pub mod traffic;
pub mod client_logs;
pub mod config_manager;
pub mod local_proxy_control;
pub mod grpc_client;
pub mod grpc_auth_provider;

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use common::protocol::control::ProxyControl;
use common::protocol::auth::ClientAuthProvider;

/// Agent Server 启动（Controller 模式，gRPC）
///
/// 通过 gRPC 双向流连接 Controller，支持断线自动重连。
pub async fn run_server_controller_mode(
    controller_url: String,
    token: String,
    bind_port: u16,
    protocol: String,
) -> Result<()> {
    // 初始化 tracing 日志系统
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx::query=warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    info!("Agent Server 启动 (Controller gRPC 模式)");
    info!("Controller: {}", controller_url);
    info!("隧道端口: {}", bind_port);
    info!("隧道协议: {}", protocol);

    // 首次连接 Controller 并认证
    let (grpc_client, cmd_rx) = grpc_client::AgentGrpcClient::connect_and_authenticate(
        &controller_url,
        &token,
        bind_port,
        &protocol,
    ).await?;

    let node_id = grpc_client.node_id().await;
    info!("连接认证成功: 节点 #{}", node_id);

    // 创建 gRPC 认证提供者（使用 SharedGrpcSender，重连后自动使用新 sender）
    let auth_provider: Arc<dyn ClientAuthProvider> = Arc::new(
        grpc_auth_provider::GrpcAuthProvider::new(&grpc_client, node_id)
    );

    // 创建 gRPC 流量管理器（使用 SharedGrpcSender，重连后自动使用新 sender）
    let traffic_manager = Arc::new(
        traffic::TrafficManager::new(grpc_client.shared_sender().clone())
    );

    // 创建配置管理器（使用默认值）
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
        auth_provider.clone(),
    ));

    // 启动首次 Controller 命令处理器
    let grpc_client_clone = grpc_client.clone();
    let proxy_control_clone = proxy_control.clone();
    tokio::spawn(async move {
        grpc_client::handle_controller_commands(cmd_rx, grpc_client_clone, proxy_control_clone).await;
    });

    // 启动隧道服务（只启动一次，不受 gRPC 重连影响）
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

    info!("所有服务已启动");

    // gRPC 断线重连监控循环
    let grpc_client_reconnect = grpc_client.clone();
    let proxy_control_reconnect = proxy_control.clone();
    let controller_url_clone = controller_url.clone();
    let token_clone = token.clone();
    let protocol_clone = protocol.clone();

    tokio::spawn(async move {
        // 等待首次连接的心跳/消息循环结束（通过检测 sender 是否可用）
        // 使用简单的轮询检测连接状态
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            // 尝试发送一个心跳来检测连接是否存活
            let test_msg = common::grpc::rfrp::AgentServerMessage {
                payload: Some(common::grpc::rfrp::agent_server_message::Payload::Heartbeat(
                    common::grpc::rfrp::Heartbeat {
                        timestamp: chrono::Utc::now().timestamp(),
                    },
                )),
            };

            if grpc_client_reconnect.shared_sender().send(test_msg).await.is_err() {
                warn!("检测到 gRPC 连接断开，开始重连...");

                loop {
                    match grpc_client_reconnect.reconnect(
                        &controller_url_clone,
                        &token_clone,
                        bind_port,
                        &protocol_clone,
                    ).await {
                        Ok(new_cmd_rx) => {
                            info!("gRPC 重连成功");

                            // 启动新的命令处理器
                            let grpc_clone = grpc_client_reconnect.clone();
                            let control_clone = proxy_control_reconnect.clone();
                            tokio::spawn(async move {
                                grpc_client::handle_controller_commands(
                                    new_cmd_rx, grpc_clone, control_clone,
                                ).await;
                            });

                            break;
                        }
                        Err(e) => {
                            error!("gRPC 重连失败: {}", e);
                            warn!("5秒后重试...");
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
    });

    // 等待终止信号
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
