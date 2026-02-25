//! gRPC Server 启动
//!
//! 在 internal_port 上启动 gRPC Server，提供 AgentServerService 和 AgentClientService。

use std::sync::Arc;
use tonic::transport::Server;
use tracing::{info, error, warn};

use common::grpc::{AgentServerServiceServer, AgentClientServiceServer};

use crate::grpc_agent_server_service::AgentServerServiceImpl;
use crate::grpc_agent_client_service::AgentClientServiceImpl;
use crate::node_manager::NodeManager;
use crate::client_stream_manager::ClientStreamManager;
use crate::config_manager::ConfigManager;

/// 启动 gRPC Server
pub fn start_grpc_server(
    port: u16,
    node_manager: Arc<NodeManager>,
    client_stream_manager: Arc<ClientStreamManager>,
    config_manager: Arc<ConfigManager>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let addr = format!("0.0.0.0:{}", port).parse().unwrap();

        let agent_server_service = AgentServerServiceImpl {
            node_manager,
        };

        let agent_client_service = AgentClientServiceImpl {
            client_stream_manager,
        };

        // 读取 TLS 配置（仅用于提示）
        let tls_enabled = config_manager.get_bool("grpc_tls_enabled", false).await;

        if tls_enabled {
            warn!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            warn!("gRPC TLS 配置已启用");
            warn!("请使用反向代理（如 Nginx/Caddy）提供 TLS 支持：");
            warn!("  1. 配置反向代理监听 HTTPS 端口");
            warn!("  2. 将流量转发到 gRPC 端口 {}", port);
            warn!("  3. 客户端使用 https:// 协议连接反向代理");
            warn!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }

        info!("gRPC Server 启动: {}", addr);

        if let Err(e) = Server::builder()
            .add_service(AgentServerServiceServer::new(agent_server_service))
            .add_service(AgentClientServiceServer::new(agent_client_service))
            .serve(addr)
            .await
        {
            error!("gRPC Server 错误: {}", e);
        }
    })
}
