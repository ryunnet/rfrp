use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_subscription")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[serde(rename = "userId")]
    pub user_id: i64,
    #[serde(rename = "subscriptionId")]
    pub subscription_id: i64,
    #[serde(rename = "startDate")]
    pub start_date: DateTime,
    #[serde(rename = "endDate")]
    pub end_date: DateTime,
    #[serde(rename = "trafficQuotaGb")]
    pub traffic_quota_gb: f64,
    #[serde(rename = "trafficUsedGb")]
    pub traffic_used_gb: f64,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "maxPortCountSnapshot")]
    pub max_port_count_snapshot: Option<i32>,
    #[serde(rename = "maxNodeCountSnapshot")]
    pub max_node_count_snapshot: Option<i32>,
    #[serde(rename = "maxClientCountSnapshot")]
    pub max_client_count_snapshot: Option<i32>,
    #[serde(rename = "quotaMerged")]
    pub quota_merged: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::subscription::Entity",
        from = "Column::SubscriptionId",
        to = "super::subscription::Column::Id"
    )]
    Subscription,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
