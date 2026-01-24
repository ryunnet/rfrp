use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set};
use std::collections::HashMap;
use tracing::{debug, error, info};
use tokio::sync::mpsc;
use std::time::Duration;

use crate::entity::{proxy, client, user, traffic_daily, Proxy, Client, User, TrafficDaily};
use crate::migration::get_connection;

struct TrafficEvent {
    proxy_id: i64,
    client_id: i64,
    user_id: Option<i64>,
    bytes_sent: i64,
    bytes_received: i64,
}

/// 流量统计管理器
#[derive(Clone)]
pub struct TrafficManager {
    sender: mpsc::Sender<TrafficEvent>,
}

impl TrafficManager {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<TrafficEvent>(10000);

        tokio::spawn(async move {
            let mut buffer: HashMap<(i64, i64, Option<i64>), (i64, i64)> = HashMap::new();
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        let key = (event.proxy_id, event.client_id, event.user_id);
                        let entry = buffer.entry(key).or_insert((0, 0));
                        entry.0 += event.bytes_sent;
                        entry.1 += event.bytes_received;

                        // 防止内存积压，如果积压太多则立即刷新
                        if buffer.len() > 1000 {
                            Self::flush_buffer(&mut buffer).await;
                        }
                    }
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            Self::flush_buffer(&mut buffer).await;
                        }
                    }
                }
            }
        });

        Self { sender: tx }
    }

    async fn flush_buffer(buffer: &mut HashMap<(i64, i64, Option<i64>), (i64, i64)>) {
        let db = get_connection().await;
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let now = Utc::now().naive_utc();

        let count = buffer.len();
        debug!("🔄 正在批量写入流量统计数据: {} 条聚合记录", count);

        for ((proxy_id, client_id, user_id), (bytes_sent, bytes_received)) in buffer.drain() {
            if bytes_sent == 0 && bytes_received == 0 {
                continue;
            }

            // 更新代理流量
            if let Ok(Some(proxy)) = Proxy::find_by_id(proxy_id).one(db).await {
                let mut proxy_active: proxy::ActiveModel = proxy.into();
                proxy_active.total_bytes_sent = Set(proxy_active.total_bytes_sent.unwrap() + bytes_sent);
                proxy_active.total_bytes_received = Set(proxy_active.total_bytes_received.unwrap() + bytes_received);
                proxy_active.updated_at = Set(now);
                if let Err(e) = proxy_active.update(db).await {
                    error!("更新代理流量失败: {}", e);
                }

                // 更新每日流量统计
                if let Ok(existing) = TrafficDaily::find()
                    .filter(traffic_daily::Column::ProxyId.eq(proxy_id))
                    .filter(traffic_daily::Column::Date.eq(&today))
                    .one(db)
                    .await
                {
                    let mut daily_active: traffic_daily::ActiveModel = existing.unwrap().into();
                    daily_active.bytes_sent = Set(daily_active.bytes_sent.unwrap() + bytes_sent);
                    daily_active.bytes_received = Set(daily_active.bytes_received.unwrap() + bytes_received);
                    daily_active.updated_at = Set(now);
                    if let Err(e) = daily_active.update(db).await {
                        error!("更新每日流量统计失败: {}", e);
                    }
                } else {
                    let daily = traffic_daily::ActiveModel {
                        id: Set(0),
                        proxy_id: Set(proxy_id),
                        client_id: Set(client_id),
                        bytes_sent: Set(bytes_sent),
                        bytes_received: Set(bytes_received),
                        date: Set(today.clone()),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    if let Err(e) = daily.insert(db).await {
                        error!("插入每日流量统计失败: {}", e);
                    }
                }
            }

            // 更新客户端流量
            if let Ok(Some(client)) = Client::find_by_id(client_id).one(db).await {
                let mut client_active: client::ActiveModel = client.into();
                client_active.total_bytes_sent = Set(client_active.total_bytes_sent.unwrap() + bytes_sent);
                client_active.total_bytes_received = Set(client_active.total_bytes_received.unwrap() + bytes_received);
                client_active.updated_at = Set(now);
                if let Err(e) = client_active.update(db).await {
                    error!("更新客户端流量失败: {}", e);
                }
            }

            // 更新用户流量
            if let Some(uid) = user_id {
                if let Ok(Some(user)) = User::find_by_id(uid).one(db).await {
                    let mut user_active: user::ActiveModel = user.into();
                    user_active.total_bytes_sent = Set(user_active.total_bytes_sent.unwrap() + bytes_sent);
                    user_active.total_bytes_received = Set(user_active.total_bytes_received.unwrap() + bytes_received);
                    user_active.updated_at = Set(now);
                    if let Err(e) = user_active.update(db).await {
                        error!("更新用户流量失败: {}", e);
                    }
                }
            }
        }
    }

    /// 实时记录流量统计到数据库 (异步非阻塞)
    pub async fn record_traffic(
        &self,
        proxy_id: i64,
        client_id: i64,
        user_id: Option<i64>,
        bytes_sent: i64,
        bytes_received: i64,
    ) {
        if bytes_sent == 0 && bytes_received == 0 {
            return;
        }

        let event = TrafficEvent {
            proxy_id,
            client_id,
            user_id,
            bytes_sent,
            bytes_received,
        };

        if let Err(e) = self.sender.send(event).await {
            error!("发送流量统计事件失败: {}", e);
        }
    }

    /// 不再需要定时刷新，保留此方法用于兼容
    pub async fn flush_to_database(&self) -> Result<()> {
        Ok(())
    }

    /// 不再需要定时刷新，保留此方法用于兼容
    pub fn start_periodic_flush(self: std::sync::Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async {})
    }
}

/// 流量统计响应结构
#[derive(Debug, serde::Serialize)]
pub struct TrafficOverview {
    pub total_traffic: TotalTraffic,
    pub by_user: Vec<UserTraffic>,
    pub by_client: Vec<ClientTraffic>,
    pub by_proxy: Vec<ProxyTraffic>,
    pub daily_traffic: Vec<DailyTraffic>,
}

#[derive(Debug, serde::Serialize)]
pub struct TotalTraffic {
    pub total_bytes_sent: i64,
    pub total_bytes_received: i64,
    pub total_bytes: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct UserTraffic {
    pub user_id: i64,
    pub username: String,
    pub total_bytes_sent: i64,
    pub total_bytes_received: i64,
    pub total_bytes: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct ClientTraffic {
    pub client_id: i64,
    pub client_name: String,
    pub total_bytes_sent: i64,
    pub total_bytes_received: i64,
    pub total_bytes: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct ProxyTraffic {
    pub proxy_id: i64,
    pub proxy_name: String,
    pub client_id: i64,
    pub client_name: String,
    pub total_bytes_sent: i64,
    pub total_bytes_received: i64,
    pub total_bytes: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct DailyTraffic {
    pub date: String,
    pub total_bytes_sent: i64,
    pub total_bytes_received: i64,
    pub total_bytes: i64,
}

/// 获取流量总览
pub async fn get_traffic_overview(user_id: Option<i64>, days: i64) -> Result<TrafficOverview> {
    let db = get_connection().await;

    let is_admin = if let Some(uid) = user_id {
        if let Some(user) = User::find_by_id(uid).one(db).await? {
            user.is_admin
        } else {
            false
        }
    } else {
        false
    };

    // 获取所有用户流量
    let mut users = Vec::new();
    let mut total_sent = 0i64;
    let mut total_received = 0i64;

    if is_admin {
        let all_users = User::find().all(db).await?;
        for user in all_users {
            let total = user.total_bytes_sent + user.total_bytes_received;
            total_sent += user.total_bytes_sent;
            total_received += user.total_bytes_received;
            users.push(UserTraffic {
                user_id: user.id,
                username: user.username,
                total_bytes_sent: user.total_bytes_sent,
                total_bytes_received: user.total_bytes_received,
                total_bytes: total,
            });
        }
    } else if let Some(uid) = user_id {
        if let Some(user) = User::find_by_id(uid).one(db).await? {
            let total = user.total_bytes_sent + user.total_bytes_received;
            total_sent += user.total_bytes_sent;
            total_received += user.total_bytes_received;
            users.push(UserTraffic {
                user_id: user.id,
                username: user.username,
                total_bytes_sent: user.total_bytes_sent,
                total_bytes_received: user.total_bytes_received,
                total_bytes: total,
            });
        }
    }

    // 获取客户端流量
    let mut clients = Vec::new();
    let all_clients = Client::find().all(db).await?;
    for client in all_clients {
        let total = client.total_bytes_sent + client.total_bytes_received;
        if !is_admin {
            // 如果不是管理员，只显示有权限的客户端
            if user_id.is_some() && !has_client_access(db, user_id.unwrap(), client.id).await? {
                continue;
            }
        }
        clients.push(ClientTraffic {
            client_id: client.id,
            client_name: client.name,
            total_bytes_sent: client.total_bytes_sent,
            total_bytes_received: client.total_bytes_received,
            total_bytes: total,
        });
    }

    // 获取代理流量
    let mut proxies = Vec::new();
    let all_proxies = Proxy::find().all(db).await?;
    for proxy in all_proxies {
        let total = proxy.total_bytes_sent + proxy.total_bytes_received;
        if !is_admin {
            // 如果不是管理员，只显示有权限的代理
            if user_id.is_some() && !has_client_access(db, user_id.unwrap(), proxy.client_id.parse::<i64>().unwrap_or(0)).await? {
                continue;
            }
        }

        let client_name = if let Some(client) = Client::find_by_id(proxy.client_id.parse::<i64>().unwrap_or(0)).one(db).await? {
            client.name
        } else {
            String::from("Unknown")
        };

        proxies.push(ProxyTraffic {
            proxy_id: proxy.id,
            proxy_name: proxy.name,
            client_id: proxy.client_id.parse::<i64>().unwrap_or(0),
            client_name,
            total_bytes_sent: proxy.total_bytes_sent,
            total_bytes_received: proxy.total_bytes_received,
            total_bytes: total,
        });
    }

    // 获取每日流量统计
    let mut daily = Vec::new();
    let start_date = Utc::now() - chrono::Duration::days(days);
    let start_date_str = start_date.format("%Y-%m-%d").to_string();

    let all_daily = TrafficDaily::find()
        .filter(traffic_daily::Column::Date.gte(&start_date_str))
        .all(db)
        .await?;

    let mut daily_map: HashMap<String, (i64, i64)> = HashMap::new();
    for d in all_daily {
        if !is_admin && user_id.is_some() {
            // 如果不是管理员，只显示有权限的代理的流量
            if !has_client_access(db, user_id.unwrap(), d.client_id).await? {
                continue;
            }
        }
        let entry = daily_map.entry(d.date.clone()).or_insert((0, 0));
        entry.0 += d.bytes_sent;
        entry.1 += d.bytes_received;
    }

    for (date, (sent, received)) in daily_map {
        daily.push(DailyTraffic {
            date,
            total_bytes_sent: sent,
            total_bytes_received: received,
            total_bytes: sent + received,
        });
    }
    daily.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(TrafficOverview {
        total_traffic: TotalTraffic {
            total_bytes_sent: total_sent,
            total_bytes_received: total_received,
            total_bytes: total_sent + total_received,
        },
        by_user: users,
        by_client: clients,
        by_proxy: proxies,
        daily_traffic: daily,
    })
}

/// 检查用户是否有访问客户端的权限
async fn has_client_access(db: &DatabaseConnection, user_id: i64, client_id: i64) -> Result<bool> {
    use crate::entity::{user_client, user_client::Entity as UserClient};

    let count = UserClient::find()
        .filter(user_client::Column::UserId.eq(user_id))
        .filter(user_client::Column::ClientId.eq(client_id))
        .count(db)
        .await?;

    Ok(count > 0)
}
