use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// 配置缓存管理器（纯内存，配置由 Controller 下发）
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

    /// 设置配置值（内存中）
    pub async fn set(&self, key: &str, value: ConfigValue) {
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), value);
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
