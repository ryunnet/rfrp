use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 为 node 表添加 public_ip 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(
                        ColumnDef::new(Node::PublicIp)
                            .string()
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
                    .table(Node::Table)
                    .drop_column(Node::PublicIp)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Node {
    Table,
    PublicIp,
}
