use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, Set, ActiveModelTrait};
use tracing::{info, warn};
use crate::entity::{SystemConfig, system_config};
use crate::migration::get_connection;

/// 配置缓存管理器
#[derive(Clone)]
pub struct ConfigManager {
    cache: Arc<RwLock<HashMap<String, ConfigValue>>>,
}

#[derive(Debug, Clone)]
pub enum ConfigValue {
    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

impl ConfigValue {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Number(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 从数据库加载所有配置到缓存
    pub async fn load_from_db(&self) -> anyhow::Result<()> {
        let db = get_connection().await;
        let configs = SystemConfig::find().all(db).await?;

        let mut cache = self.cache.write().await;
        for config in configs {
            let value = self.parse_value(&config.value, &config.value_type);
            cache.insert(config.key.clone(), value);
        }

        info!("✅ 已加载 {} 个系统配置项", cache.len());
        Ok(())
    }

    /// 获取配置值
    pub async fn get(&self, key: &str) -> Option<ConfigValue> {
        let cache = self.cache.read().await;
        cache.get(key).cloned()
    }

    /// 获取数值配置（带默认值）
    pub async fn get_number(&self, key: &str, default: i64) -> i64 {
        self.get(key).await
            .and_then(|v| v.as_i64())
            .unwrap_or(default)
    }

    /// 获取浮点配置（带默认值）
    pub async fn get_float(&self, key: &str, default: f64) -> f64 {
        self.get(key).await
            .and_then(|v| v.as_f64())
            .unwrap_or(default)
    }

    /// 获取字符串配置（带默认值）
    pub async fn get_string(&self, key: &str, default: &str) -> String {
        self.get(key).await
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| default.to_string())
    }

    /// 获取布尔配置（带默认值）
    pub async fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get(key).await
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// 更新配置值
    pub async fn set(&self, key: &str, value: ConfigValue) -> anyhow::Result<()> {
        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), value.clone());
        }

        // 更新数据库
        let db = get_connection().await;

        let value_str = match &value {
            ConfigValue::Number(n) => n.to_string(),
            ConfigValue::Float(f) => f.to_string(),
            ConfigValue::String(s) => serde_json::to_string(s)?,
            ConfigValue::Boolean(b) => b.to_string(),
        };

        if let Some(config) = SystemConfig::find()
            .filter(system_config::Column::Key.eq(key))
            .one(db)
            .await?
        {
            let mut active_model: system_config::ActiveModel = config.into();
            active_model.value = Set(value_str);
            active_model.updated_at = Set(chrono::Utc::now().naive_utc());
            active_model.update(db).await?;
        }

        Ok(())
    }

    /// 解析配置值
    fn parse_value(&self, value_str: &str, value_type: &str) -> ConfigValue {
        match value_type {
            "number" => {
                if let Ok(n) = value_str.parse::<i64>() {
                    ConfigValue::Number(n)
                } else if let Ok(f) = value_str.parse::<f64>() {
                    ConfigValue::Float(f)
                } else {
                    warn!("无法解析数值配置: {}", value_str);
                    ConfigValue::Number(0)
                }
            }
            "boolean" => {
                ConfigValue::Boolean(value_str.parse::<bool>().unwrap_or(false))
            }
            "string" => {
                // 尝试解析 JSON 字符串
                if let Ok(s) = serde_json::from_str::<String>(value_str) {
                    ConfigValue::String(s)
                } else {
                    ConfigValue::String(value_str.to_string())
                }
            }
            _ => ConfigValue::String(value_str.to_string()),
        }
    }

    /// 重新加载配置（用于配置更新后刷新）
    pub async fn reload(&self) -> anyhow::Result<()> {
        self.load_from_db().await
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
