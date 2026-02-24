use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "subscription")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    #[serde(rename = "durationType")]
    pub duration_type: String, // daily, weekly, monthly, yearly
    #[serde(rename = "durationValue")]
    pub duration_value: i32,
    #[serde(rename = "trafficQuotaGb")]
    pub traffic_quota_gb: f64,
    pub price: Option<f64>,
    pub description: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_subscription::Entity")]
    UserSubscriptions,
}

impl Related<super::user_subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSubscriptions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
