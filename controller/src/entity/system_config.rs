use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "system_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 配置键名
    pub key: String,
    /// 配置值（JSON格式）
    pub value: String,
    /// 配置说明
    pub description: String,
    /// 配置类型：number, string, boolean
    pub value_type: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// 系统配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfigItem {
    pub id: i64,
    pub key: String,
    pub value: serde_json::Value,
    pub description: String,
    #[serde(rename = "valueType")]
    pub value_type: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
}

impl From<Model> for SystemConfigItem {
    fn from(model: Model) -> Self {
        let value = serde_json::from_str(&model.value).unwrap_or(serde_json::Value::Null);
        Self {
            id: model.id,
            key: model.key,
            value,
            description: model.description,
            value_type: model.value_type,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// 配置键常量
pub mod config_keys {
    /// 心跳检查间隔（秒）
    pub const HEALTH_CHECK_INTERVAL: &str = "health_check_interval";
    /// 空闲超时时间（秒）
    pub const IDLE_TIMEOUT: &str = "idle_timeout";
    /// Keep-Alive 心跳间隔（秒）
    pub const KEEP_ALIVE_INTERVAL: &str = "keep_alive_interval";
    /// 最大并发流数量
    pub const MAX_CONCURRENT_STREAMS: &str = "max_concurrent_streams";
}
