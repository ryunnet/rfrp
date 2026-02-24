use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 为 user 表添加流量配额字段
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::TrafficQuotaGb)
                            .double()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        // 为 client 表添加流量配额字段
        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .add_column(
                        ColumnDef::new(Client::TrafficQuotaGb)
                            .double()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::TrafficQuotaGb)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .drop_column(Client::TrafficQuotaGb)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    TrafficQuotaGb,
}

#[derive(DeriveIden)]
enum Client {
    Table,
    TrafficQuotaGb,
}
