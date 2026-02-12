//! 服务端配置模块

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tokio::sync::OnceCell;

// 从共享库导入 KcpConfig
pub use rfrp_common::KcpConfig;

/// 服务端配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// QUIC 绑定端口
    pub bind_port: u16,

    /// JWT 密钥 (可选，默认从环境变量 JWT_SECRET 读取)
    #[serde(default)]
    pub jwt_secret: Option<String>,

    /// JWT 过期时间（小时）
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration_hours: i64,

    /// Web 管理界面端口
    #[serde(default = "default_web_port")]
    pub web_port: u16,

    /// 数据库路径
    #[serde(default = "default_db_path")]
    pub db_path: String,
}

fn default_jwt_expiration() -> i64 {
    24
}

fn default_web_port() -> u16 {
    3000
}

fn default_db_path() -> String {
    "./data/rfrp.db".to_string()
}

impl Config {
    /// 获取绑定地址字符串
    pub fn get_bind_addr(&self) -> String {
        format!("0.0.0.0:{}", self.bind_port)
    }

    /// 获取 JWT 密钥（优先从环境变量读取，其次从配置文件，最后自动生成）
    pub fn get_jwt_secret(&self) -> anyhow::Result<String> {
        // 优先从环境变量读取
        if let Ok(secret) = std::env::var("JWT_SECRET") {
            if !secret.is_empty() {
                return Ok(secret);
            }
        }

        // 其次从配置文件读取
        if let Some(ref secret) = self.jwt_secret {
            if !secret.is_empty() {
                return Ok(secret.clone());
            }
        }

        // 如果都没有，从持久化文件读取或生成新密钥
        Self::get_or_generate_jwt_secret()
    }

    /// 从文件获取或生成新的 JWT 密钥
    fn get_or_generate_jwt_secret() -> anyhow::Result<String> {
        use std::path::PathBuf;

        let data_dir = PathBuf::from("./data");
        let secret_file = data_dir.join("jwt_secret.key");

        // 尝试从文件读取
        if secret_file.exists() {
            if let Ok(secret) = fs::read_to_string(&secret_file) {
                let secret = secret.trim();
                if !secret.is_empty() {
                    return Ok(secret.to_string());
                }
            }
        }

        // 文件不存在或读取失败，生成新密钥
        let secret = Self::generate_random_secret(64);

        // 确保 data 目录存在
        if let Err(e) = fs::create_dir_all(&data_dir) {
            tracing::warn!("无法创建 data 目录: {}", e);
        } else {
            // 保存密钥到文件
            if let Err(e) = fs::write(&secret_file, &secret) {
                tracing::warn!("无法保存 JWT 密钥到文件: {}", e);
            } else {
                tracing::info!("🔑 已生成并保存新的 JWT 密钥到: {}", secret_file.display());
            }
        }

        Ok(secret)
    }

    /// 生成随机密钥
    fn generate_random_secret(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

static CONFIG: OnceCell<Config> = OnceCell::const_new();

/// 获取全局配置
pub async fn get_config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}

/// 初始化配置
pub async fn init_config() -> Config {
    let path = Path::new("rfrps.toml");
    let content = fs::read_to_string(path)
        .with_context(|| format!("无法读取配置文件: {}", path.display()))
        .unwrap();

    let config: Config = toml::from_str(&content)
        .with_context(|| "解析配置文件失败")
        .unwrap();
    config
}
