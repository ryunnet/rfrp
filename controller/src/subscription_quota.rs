use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::entity::{User, UserSubscription, user_subscription};

/// 用户套餐配额信息（仅用于展示）
#[derive(Debug, Clone)]
pub struct UserSubscriptionQuota {
    /// 总流量配额（GB）
    pub total_traffic_quota_gb: f64,
    /// 总端口数量限制
    pub total_max_port_count: Option<i32>,
    /// 总节点数量限制
    pub total_max_node_count: Option<i32>,
    /// 总客户端数量限制
    pub total_max_client_count: Option<i32>,
}

/// 获取用户所有激活套餐的累加配额（仅用于展示，不参与配额计算）
pub async fn get_user_subscription_quota(
    user_id: i64,
    db: &DatabaseConnection,
) -> Result<UserSubscriptionQuota> {
    let user_subscriptions = UserSubscription::find()
        .filter(user_subscription::Column::UserId.eq(user_id))
        .filter(user_subscription::Column::IsActive.eq(true))
        .find_also_related(crate::entity::Subscription)
        .all(db)
        .await?;

    let mut total_traffic_quota_gb = 0.0;
    let mut total_port_count: Option<i32> = None;
    let mut total_node_count: Option<i32> = None;
    let mut total_client_count: Option<i32> = None;

    for (user_sub, subscription_opt) in user_subscriptions {
        total_traffic_quota_gb += user_sub.traffic_quota_gb;

        if let Some(subscription) = subscription_opt {
            if let Some(port_count) = subscription.max_port_count {
                total_port_count = Some(total_port_count.unwrap_or(0) + port_count);
            }
            if let Some(node_count) = subscription.max_node_count {
                total_node_count = Some(total_node_count.unwrap_or(0) + node_count);
            }
            if let Some(client_count) = subscription.max_client_count {
                total_client_count = Some(total_client_count.unwrap_or(0) + client_count);
            }
        }
    }

    Ok(UserSubscriptionQuota {
        total_traffic_quota_gb,
        total_max_port_count: total_port_count,
        total_max_node_count: total_node_count,
        total_max_client_count: total_client_count,
    })
}

/// 获取用户的最终配额
///
/// 套餐配额已物理合并到用户字段中，直接返回用户字段即可。
pub async fn get_user_final_quota(
    _user_id: i64,
    user_traffic_quota_gb: Option<f64>,
    user_max_port_count: Option<i32>,
    user_max_node_count: Option<i32>,
    user_max_client_count: Option<i32>,
    _db: &DatabaseConnection,
) -> Result<(Option<f64>, Option<i32>, Option<i32>, Option<i32>)> {
    Ok((user_traffic_quota_gb, user_max_port_count, user_max_node_count, user_max_client_count))
}

/// 将套餐配额合并到用户字段
pub async fn merge_subscription_quota_to_user(
    user_id: i64,
    traffic_quota_gb: f64,
    max_port_count: Option<i32>,
    max_node_count: Option<i32>,
    max_client_count: Option<i32>,
    db: &DatabaseConnection,
) -> Result<()> {
    let user = User::find_by_id(user_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户 #{} 不存在", user_id))?;

    let mut active: crate::entity::user::ActiveModel = user.clone().into();

    // 合并流量配额
    let current_traffic = user.traffic_quota_gb.unwrap_or(0.0);
    active.traffic_quota_gb = Set(Some(current_traffic + traffic_quota_gb));

    // 合并端口数量
    if let Some(port_count) = max_port_count {
        let current = user.max_port_count.unwrap_or(0);
        active.max_port_count = Set(Some(current + port_count));
    }

    // 合并节点数量
    if let Some(node_count) = max_node_count {
        let current = user.max_node_count.unwrap_or(0);
        active.max_node_count = Set(Some(current + node_count));
    }

    // 合并客户端数量
    if let Some(client_count) = max_client_count {
        let current = user.max_client_count.unwrap_or(0);
        active.max_client_count = Set(Some(current + client_count));
    }

    active.updated_at = Set(chrono::Utc::now().naive_utc());
    active.update(db).await?;

    Ok(())
}

/// 从用户字段中回退套餐配额
///
/// 所有字段下限为 0，不会设为 None（None 表示无限制）。
pub async fn rollback_subscription_quota_from_user(
    user_id: i64,
    traffic_quota_gb: f64,
    max_port_count: Option<i32>,
    max_node_count: Option<i32>,
    max_client_count: Option<i32>,
    db: &DatabaseConnection,
) -> Result<()> {
    let user = User::find_by_id(user_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户 #{} 不存在", user_id))?;

    let mut active: crate::entity::user::ActiveModel = user.clone().into();

    // 回退流量配额，下限 0.0
    let current_traffic = user.traffic_quota_gb.unwrap_or(0.0);
    active.traffic_quota_gb = Set(Some((current_traffic - traffic_quota_gb).max(0.0)));

    // 回退端口数量，下限 0
    if let Some(port_count) = max_port_count {
        let current = user.max_port_count.unwrap_or(0);
        active.max_port_count = Set(Some((current - port_count).max(0)));
    }

    // 回退节点数量，下限 0
    if let Some(node_count) = max_node_count {
        let current = user.max_node_count.unwrap_or(0);
        active.max_node_count = Set(Some((current - node_count).max(0)));
    }

    // 回退客户端数量，下限 0
    if let Some(client_count) = max_client_count {
        let current = user.max_client_count.unwrap_or(0);
        active.max_client_count = Set(Some((current - client_count).max(0)));
    }

    active.updated_at = Set(chrono::Utc::now().naive_utc());
    active.update(db).await?;

    Ok(())
}

/// 过期所有已到期的激活订阅，回退配额并设为非激活
pub async fn expire_subscriptions(db: &DatabaseConnection) -> Result<Vec<(i64, i64)>> {
    let now = chrono::Utc::now().naive_utc();

    let expired_subs = UserSubscription::find()
        .filter(user_subscription::Column::IsActive.eq(true))
        .filter(user_subscription::Column::EndDate.lt(now))
        .all(db)
        .await?;

    let mut expired_list = Vec::new();

    for sub in expired_subs {
        let sub_id = sub.id;
        let user_id = sub.user_id;

        // 回退已合并的配额
        if sub.quota_merged {
            if let Err(e) = rollback_subscription_quota_from_user(
                sub.user_id,
                sub.traffic_quota_gb,
                sub.max_port_count_snapshot,
                sub.max_node_count_snapshot,
                sub.max_client_count_snapshot,
                db,
            )
            .await
            {
                tracing::error!("回滚过期订阅 #{} 配额失败: {}", sub_id, e);
                continue;
            }
        }

        // 标记为非激活
        let mut active_sub: user_subscription::ActiveModel = sub.into();
        active_sub.is_active = Set(false);
        active_sub.updated_at = Set(now);
        if let Err(e) = active_sub.update(db).await {
            tracing::error!("更新过期订阅 #{} 状态失败: {}", sub_id, e);
            continue;
        }

        expired_list.push((sub_id, user_id));
    }

    Ok(expired_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_calculation() {
        // 测试配额计算逻辑
        // 注意：这里只是示例，实际测试需要数据库连接
    }
}
