use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 为 client 表添加 user_id 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .add_column(ColumnDef::new(Client::UserId).big_integer().null())
                    .to_owned(),
            )
            .await?;

        // 添加索引（SQLite 不支持通过 ALTER TABLE 添加外键约束）
        manager
            .create_index(
                Index::create()
                    .name("idx_client_user_id")
                    .table(Client::Table)
                    .col(Client::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_client_user_id")
                    .table(Client::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .drop_column(Client::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Client {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
