use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub bind_port: u16,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件: {}", path_ref.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| "解析配置文件失败")?;

        Ok(config)
    }

    pub fn get_bind_addr(&self) -> String {
        format!("0.0.0.0:{}", self.bind_port)
    }
}
