use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "client")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub token: String,
    pub is_online: bool,
    #[serde(rename = "totalBytesSent")]
    pub total_bytes_sent: i64,
    #[serde(rename = "totalBytesReceived")]
    pub total_bytes_received: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_client::Entity")]
    UserClients,
}

impl Related<super::user_client::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserClients.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
