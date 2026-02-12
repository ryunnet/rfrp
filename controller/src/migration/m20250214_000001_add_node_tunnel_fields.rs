use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 给 node 表添加隧道连接字段
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(
                        ColumnDef::new(Node::TunnelAddr)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(
                        ColumnDef::new(Node::TunnelPort)
                            .integer()
                            .not_null()
                            .default(7000),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(
                        ColumnDef::new(Node::TunnelProtocol)
                            .string()
                            .not_null()
                            .default("quic"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(
                        ColumnDef::new(Node::KcpConfig)
                            .text()
                            .null(),
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
                    .drop_column(Node::KcpConfig)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .drop_column(Node::TunnelProtocol)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .drop_column(Node::TunnelPort)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .drop_column(Node::TunnelAddr)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Node {
    Table,
    TunnelAddr,
    TunnelPort,
    TunnelProtocol,
    KcpConfig,
}
