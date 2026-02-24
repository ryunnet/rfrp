pub mod pending_requests;

// 导出 proto 生成的代码
pub mod rfrp {
    tonic::include_proto!("rfrp");
}

// 重新导出常用类型
pub use rfrp::*;
pub use rfrp::agent_server_service_client::AgentServerServiceClient;
pub use rfrp::agent_server_service_server::{AgentServerService, AgentServerServiceServer};
pub use rfrp::agent_client_service_client::AgentClientServiceClient;
pub use rfrp::agent_client_service_server::{AgentClientService, AgentClientServiceServer};
