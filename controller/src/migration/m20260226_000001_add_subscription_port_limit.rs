use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 为 subscription 表添加 max_port_count 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Subscription::Table)
                    .add_column(
                        ColumnDef::new(Subscription::MaxPortCount)
                            .integer()
                            .null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚：删除 max_port_count 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Subscription::Table)
                    .drop_column(Subscription::MaxPortCount)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Subscription {
    Table,
    MaxPortCount,
}
