//! gRPC Server 启动
//!
//! 在 internal_port 上启动 gRPC Server，提供 AgentServerService 和 AgentClientService。

use std::sync::Arc;
use tonic::transport::Server;
use tracing::{info, error};

use common::grpc::{AgentServerServiceServer, AgentClientServiceServer};

use crate::grpc_agent_server_service::AgentServerServiceImpl;
use crate::grpc_agent_client_service::AgentClientServiceImpl;
use crate::node_manager::NodeManager;
use crate::client_stream_manager::ClientStreamManager;

/// 启动 gRPC Server
pub fn start_grpc_server(
    port: u16,
    node_manager: Arc<NodeManager>,
    client_stream_manager: Arc<ClientStreamManager>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let addr = format!("0.0.0.0:{}", port).parse().unwrap();

        let agent_server_service = AgentServerServiceImpl {
            node_manager,
        };

        let agent_client_service = AgentClientServiceImpl {
            client_stream_manager,
        };

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
