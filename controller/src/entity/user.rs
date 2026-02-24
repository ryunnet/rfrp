use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    #[serde(rename = "totalBytesSent")]
    pub total_bytes_sent: i64,
    #[serde(rename = "totalBytesReceived")]
    pub total_bytes_received: i64,
    #[serde(rename = "trafficResetCycle")]
    pub traffic_reset_cycle: String,
    #[serde(rename = "lastResetAt")]
    pub last_reset_at: Option<DateTime>,
    #[serde(rename = "isTrafficExceeded")]
    pub is_traffic_exceeded: bool,
    #[serde(rename = "trafficQuotaGb")]
    pub traffic_quota_gb: Option<f64>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_client::Entity")]
    UserClients,
    #[sea_orm(has_many = "super::user_node::Entity")]
    UserNodes,
}

impl Related<super::user_client::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserClients.def()
    }
}

impl Related<super::user_node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserNodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
