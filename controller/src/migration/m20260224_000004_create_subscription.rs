use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 subscription 表（订阅套餐）
        manager
            .create_table(
                Table::create()
                    .table(Subscription::Table)
                    .if_not_exists()
                    .col(big_integer(Subscription::Id).auto_increment().primary_key())
                    .col(string(Subscription::Name))
                    .col(string(Subscription::DurationType)) // daily, weekly, monthly, yearly
                    .col(integer(Subscription::DurationValue).default(1))
                    .col(double(Subscription::TrafficQuotaGb))
                    .col(double(Subscription::Price).null())
                    .col(string(Subscription::Description).null())
                    .col(boolean(Subscription::IsActive).default(true))
                    .col(timestamp(Subscription::CreatedAt))
                    .col(timestamp(Subscription::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 创建 user_subscription 表（用户订阅记录）
        manager
            .create_table(
                Table::create()
                    .table(UserSubscription::Table)
                    .if_not_exists()
                    .col(big_integer(UserSubscription::Id).auto_increment().primary_key())
                    .col(big_integer(UserSubscription::UserId))
                    .col(big_integer(UserSubscription::SubscriptionId))
                    .col(timestamp(UserSubscription::StartDate))
                    .col(timestamp(UserSubscription::EndDate))
                    .col(double(UserSubscription::TrafficQuotaGb))
                    .col(double(UserSubscription::TrafficUsedGb).default(0.0))
                    .col(boolean(UserSubscription::IsActive).default(true))
                    .col(timestamp(UserSubscription::CreatedAt))
                    .col(timestamp(UserSubscription::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_subscription_user")
                            .from(UserSubscription::Table, UserSubscription::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_subscription_subscription")
                            .from(UserSubscription::Table, UserSubscription::SubscriptionId)
                            .to(Subscription::Table, Subscription::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 为 user_id 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_user_subscription_user_id")
                    .table(UserSubscription::Table)
                    .col(UserSubscription::UserId)
                    .to_owned(),
            )
            .await?;

        // 为 subscription_id 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_user_subscription_subscription_id")
                    .table(UserSubscription::Table)
                    .col(UserSubscription::SubscriptionId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserSubscription::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Subscription::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Subscription {
    Table,
    Id,
    Name,
    DurationType,
    DurationValue,
    TrafficQuotaGb,
    Price,
    Description,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserSubscription {
    Table,
    Id,
    UserId,
    SubscriptionId,
    StartDate,
    EndDate,
    TrafficQuotaGb,
    TrafficUsedGb,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
