use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 为 subscription 表添加 max_node_count 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Subscription::Table)
                    .add_column(
                        ColumnDef::new(Subscription::MaxNodeCount)
                            .integer()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        // 为 subscription 表添加 max_client_count 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Subscription::Table)
                    .add_column(
                        ColumnDef::new(Subscription::MaxClientCount)
                            .integer()
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
                    .table(Subscription::Table)
                    .drop_column(Subscription::MaxNodeCount)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Subscription::Table)
                    .drop_column(Subscription::MaxClientCount)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Subscription {
    Table,
    MaxNodeCount,
    MaxClientCount,
}
