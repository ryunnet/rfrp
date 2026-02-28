//! IP 地理位置查询服务
//!
//! 使用免费的 IP 地理位置 API 查询 IP 地址的地理位置信息

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// IP 地理位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoIpInfo {
    pub ip: String,
    pub region: String,
}

/// 从 ipwhois.app 查询地理位置信息
#[derive(Debug, Deserialize)]
struct IpWhoisResponse {
    ip: Option<String>,
    success: Option<bool>,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
}

/// 查询 IP 地址的地理位置信息
/// 使用 ipwhois.app 免费服务（每月 10k 次请求，支持中文）
pub async fn query_geo_ip(ip: &str) -> Result<GeoIpInfo> {
    let url = format!(
        "https://ipwhois.app/json/{}?lang=zh-CN&objects=ip,success,country,region,city",
        ip
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("请求 IP 地理位置 API 失败: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("IP 地理位置 API 返回错误状态: {}", response.status()));
    }

    let api_response: IpWhoisResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("解析 IP 地理位置响应失败: {}", e))?;

    if api_response.success == Some(false) {
        return Err(anyhow!("IP 地理位置查询失败"));
    }

    // 构建地区字符串：国家-省份-城市
    let mut region_parts = Vec::new();

    if let Some(country) = api_response.country {
        if !country.is_empty() {
            region_parts.push(country);
        }
    }

    if let Some(region) = api_response.region {
        if !region.is_empty() {
            region_parts.push(region);
        }
    }

    if let Some(city) = api_response.city {
        if !city.is_empty() {
            region_parts.push(city);
        }
    }

    let region = if region_parts.is_empty() {
        "Unknown".to_string()
    } else {
        region_parts.join("-")
    };

    let ip = api_response.ip.unwrap_or_else(|| ip.to_string());

    info!("查询到 IP {} 的地理位置: {}", ip, region);

    Ok(GeoIpInfo { ip, region })
}

/// 从 gRPC 连接中提取客户端 IP 地址
pub fn extract_client_ip_from_request<T>(request: &tonic::Request<T>) -> Option<String> {
    // 尝试从 metadata 中获取真实 IP（如果有反向代理）
    if let Some(forwarded) = request.metadata().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // X-Forwarded-For 可能包含多个 IP，取第一个
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }

    // 从 remote_addr 获取
    if let Some(remote_addr) = request.remote_addr() {
        return Some(remote_addr.ip().to_string());
    }

    None
}

/// 查询节点的公网 IP 和地理位置
/// 如果提供了 IP 地址，则查询该 IP；否则查询本机公网 IP
pub async fn query_node_geo_info(ip: Option<String>) -> Result<GeoIpInfo> {
    let target_ip = if let Some(ip) = ip {
        ip
    } else {
        // 查询本机公网 IP
        get_public_ip().await?
    };

    query_geo_ip(&target_ip).await
}

/// 获取本机公网 IP 地址
async fn get_public_ip() -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    // 使用多个服务作为备选
    let services = vec![
        "https://api.ipify.org",
        "https://ifconfig.me/ip",
        "https://icanhazip.com",
    ];

    for service in services {
        match client.get(service).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(ip) = response.text().await {
                        let ip = ip.trim().to_string();
                        if !ip.is_empty() {
                            info!("获取到公网 IP: {}", ip);
                            return Ok(ip);
                        }
                    }
                }
            }
            Err(e) => {
                error!("从 {} 获取公网 IP 失败: {}", service, e);
                continue;
            }
        }
    }

    Err(anyhow!("无法获取公网 IP 地址"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_geo_ip() {
        // 测试查询谷歌 DNS 的地理位置
        let result = query_geo_ip("8.8.8.8").await;
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.ip, "8.8.8.8");
        assert!(!info.region.is_empty());
        println!("8.8.8.8 的地理位置: {}", info.region);
    }
}
