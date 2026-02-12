use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 node 表
        manager
            .create_table(
                Table::create()
                    .table(Node::Table)
                    .if_not_exists()
                    .col(big_integer(Node::Id).auto_increment().primary_key())
                    .col(string(Node::Name))
                    .col(string(Node::Url))
                    .col(string(Node::Secret))
                    .col(boolean(Node::IsOnline).default(false))
                    .col(string_null(Node::Region))
                    .col(string_null(Node::Description))
                    .col(timestamp(Node::CreatedAt))
                    .col(timestamp(Node::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 给 client 表添加 node_id 列
        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .add_column(big_integer_null(Client::NodeId))
                    .to_owned(),
            )
            .await?;

        // 创建 client.node_id 索引
        manager
            .create_index(
                Index::create()
                    .name("idx_client_node_id")
                    .table(Client::Table)
                    .col(Client::NodeId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除 client.node_id 索引
        manager
            .drop_index(
                Index::drop()
                    .name("idx_client_node_id")
                    .table(Client::Table)
                    .to_owned(),
            )
            .await?;

        // 删除 client.node_id 列
        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .drop_column(Client::NodeId)
                    .to_owned(),
            )
            .await?;

        // 删除 node 表
        manager
            .drop_table(Table::drop().table(Node::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Node {
    Table,
    Id,
    Name,
    Url,
    Secret,
    IsOnline,
    Region,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Client {
    Table,
    NodeId,
}
