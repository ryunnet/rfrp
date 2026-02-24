use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub url: String,
    pub secret: String,
    #[serde(rename = "isOnline")]
    pub is_online: bool,
    pub region: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "tunnelAddr")]
    pub tunnel_addr: String,
    #[serde(rename = "tunnelPort")]
    pub tunnel_port: i32,
    #[serde(rename = "tunnelProtocol")]
    pub tunnel_protocol: String,
    #[serde(rename = "kcpConfig")]
    pub kcp_config: Option<String>,
    #[serde(rename = "nodeType")]
    pub node_type: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_node::Entity")]
    UserNodes,
}

impl Related<super::user_node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserNodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
