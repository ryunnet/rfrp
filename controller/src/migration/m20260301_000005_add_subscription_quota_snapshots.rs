use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 为 user_subscription 表添加配额快照字段
        manager
            .alter_table(
                Table::alter()
                    .table(UserSubscription::Table)
                    .add_column(ColumnDef::new(UserSubscription::MaxPortCountSnapshot).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserSubscription::Table)
                    .add_column(ColumnDef::new(UserSubscription::MaxNodeCountSnapshot).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserSubscription::Table)
                    .add_column(ColumnDef::new(UserSubscription::MaxClientCountSnapshot).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserSubscription::Table)
                    .add_column(
                        ColumnDef::new(UserSubscription::QuotaMerged)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // 数据迁移：为已有的激活订阅填充快照字段
        let db = manager.get_connection();
        db.execute_unprepared(
            "UPDATE user_subscription SET \
             max_port_count_snapshot = (SELECT max_port_count FROM subscription WHERE subscription.id = user_subscription.subscription_id), \
             max_node_count_snapshot = (SELECT max_node_count FROM subscription WHERE subscription.id = user_subscription.subscription_id), \
             max_client_count_snapshot = (SELECT max_client_count FROM subscription WHERE subscription.id = user_subscription.subscription_id), \
             quota_merged = 1 \
             WHERE is_active = 1"
        ).await?;

        // 将已有激活订阅的端口额度合并到用户（traffic 已由旧代码合并，不重复处理）
        db.execute_unprepared(
            "UPDATE user SET max_port_count = COALESCE(max_port_count, 0) + COALESCE( \
             (SELECT SUM(max_port_count_snapshot) FROM user_subscription \
              WHERE user_subscription.user_id = user.id AND user_subscription.is_active = 1 \
              AND user_subscription.max_port_count_snapshot IS NOT NULL), 0) \
             WHERE id IN (SELECT DISTINCT user_id FROM user_subscription WHERE is_active = 1 AND max_port_count_snapshot IS NOT NULL)"
        ).await?;

        db.execute_unprepared(
            "UPDATE user SET max_node_count = COALESCE(max_node_count, 0) + COALESCE( \
             (SELECT SUM(max_node_count_snapshot) FROM user_subscription \
              WHERE user_subscription.user_id = user.id AND user_subscription.is_active = 1 \
              AND user_subscription.max_node_count_snapshot IS NOT NULL), 0) \
             WHERE id IN (SELECT DISTINCT user_id FROM user_subscription WHERE is_active = 1 AND max_node_count_snapshot IS NOT NULL)"
        ).await?;

        db.execute_unprepared(
            "UPDATE user SET max_client_count = COALESCE(max_client_count, 0) + COALESCE( \
             (SELECT SUM(max_client_count_snapshot) FROM user_subscription \
              WHERE user_subscription.user_id = user.id AND user_subscription.is_active = 1 \
              AND user_subscription.max_client_count_snapshot IS NOT NULL), 0) \
             WHERE id IN (SELECT DISTINCT user_id FROM user_subscription WHERE is_active = 1 AND max_client_count_snapshot IS NOT NULL)"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserSubscription::Table)
                    .drop_column(UserSubscription::MaxPortCountSnapshot)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserSubscription::Table)
                    .drop_column(UserSubscription::MaxNodeCountSnapshot)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserSubscription::Table)
                    .drop_column(UserSubscription::MaxClientCountSnapshot)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserSubscription::Table)
                    .drop_column(UserSubscription::QuotaMerged)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserSubscription {
    Table,
    MaxPortCountSnapshot,
    MaxNodeCountSnapshot,
    MaxClientCountSnapshot,
    QuotaMerged,
}
