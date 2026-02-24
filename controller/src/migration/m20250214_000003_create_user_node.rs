use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 user_node 表 (用户-节点关联表)
        manager
            .create_table(
                Table::create()
                    .table(UserNode::Table)
                    .if_not_exists()
                    .col(big_integer(UserNode::Id).auto_increment().primary_key())
                    .col(big_integer(UserNode::UserId))
                    .col(big_integer(UserNode::NodeId))
                    .col(timestamp(UserNode::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_node_user")
                            .from(UserNode::Table, UserNode::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_node_node")
                            .from(UserNode::Table, UserNode::NodeId)
                            .to(Node::Table, Node::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建唯一索引 (user_id, node_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_user_node_unique")
                    .table(UserNode::Table)
                    .col(UserNode::UserId)
                    .col(UserNode::NodeId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserNode::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserNode {
    Table,
    Id,
    UserId,
    NodeId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Node {
    Table,
    Id,
}
