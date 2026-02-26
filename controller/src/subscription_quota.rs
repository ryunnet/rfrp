use anyhow::Result;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::entity::{UserSubscription, user_subscription};

/// 用户套餐配额信息
#[derive(Debug, Clone)]
pub struct UserSubscriptionQuota {
    /// 总流量配额（GB）
    pub total_traffic_quota_gb: f64,
    /// 总端口数量限制
    pub total_max_port_count: Option<i32>,
}

/// 获取用户所有激活套餐的累加配额
///
/// 计算逻辑：
/// - 流量配额：累加所有激活套餐的 traffic_quota_gb
/// - 端口数量：累加所有激活套餐的 max_port_count（如果套餐设置了端口限制）
pub async fn get_user_subscription_quota(
    user_id: i64,
    db: &DatabaseConnection,
) -> Result<UserSubscriptionQuota> {
    // 查询用户所有激活的套餐
    let user_subscriptions = UserSubscription::find()
        .filter(user_subscription::Column::UserId.eq(user_id))
        .filter(user_subscription::Column::IsActive.eq(true))
        .find_also_related(crate::entity::Subscription)
        .all(db)
        .await?;

    let mut total_traffic_quota_gb = 0.0;
    let mut total_port_count: Option<i32> = None;

    for (user_sub, subscription_opt) in user_subscriptions {
        // 累加流量配额
        total_traffic_quota_gb += user_sub.traffic_quota_gb;

        // 累加端口数量（如果套餐设置了端口限制）
        if let Some(subscription) = subscription_opt {
            if let Some(port_count) = subscription.max_port_count {
                total_port_count = Some(total_port_count.unwrap_or(0) + port_count);
            }
        }
    }

    Ok(UserSubscriptionQuota {
        total_traffic_quota_gb,
        total_max_port_count: total_port_count,
    })
}

/// 获取用户的最终配额（用户直接配额 + 套餐配额）
///
/// 优先级：
/// 1. 如果用户有套餐配额，使用套餐配额
/// 2. 如果用户没有套餐配额，使用用户表中的直接配额
/// 3. 如果都没有，返回 None
pub async fn get_user_final_quota(
    user_id: i64,
    user_traffic_quota_gb: Option<f64>,
    user_max_port_count: Option<i32>,
    db: &DatabaseConnection,
) -> Result<(Option<f64>, Option<i32>)> {
    // 获取套餐配额
    let subscription_quota = get_user_subscription_quota(user_id, db).await?;

    // 流量配额：优先使用套餐配额，如果套餐配额为0则使用用户直接配额
    let final_traffic_quota = if subscription_quota.total_traffic_quota_gb > 0.0 {
        Some(subscription_quota.total_traffic_quota_gb)
    } else {
        user_traffic_quota_gb
    };

    // 端口配额：优先使用套餐配额，如果套餐没有设置则使用用户直接配额
    let final_port_count = subscription_quota.total_max_port_count.or(user_max_port_count);

    Ok((final_traffic_quota, final_port_count))
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
