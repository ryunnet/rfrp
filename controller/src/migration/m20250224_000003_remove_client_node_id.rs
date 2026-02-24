use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 先删除索引
        manager
            .drop_index(
                Index::drop()
                    .name("idx_client_node_id")
                    .table(Client::Table)
                    .to_owned(),
            )
            .await?;

        // 然后删除 client 表的 node_id 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .drop_column(Client::NodeId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚：先重新添加 node_id 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .add_column(
                        ColumnDef::new(Client::NodeId)
                            .big_integer()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        // 然后重新创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_client_node_id")
                    .table(Client::Table)
                    .col(Client::NodeId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Client {
    Table,
    NodeId,
}
