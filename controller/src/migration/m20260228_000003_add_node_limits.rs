use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 最大代理数量（NULL=不限）
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(ColumnDef::new(Node::MaxProxyCount).integer().null())
                    .to_owned(),
            )
            .await?;

        // 允许的端口范围，格式: "1000-9999,20000-30000"
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(ColumnDef::new(Node::AllowedPortRange).string().null())
                    .to_owned(),
            )
            .await?;

        // 流量配额(GB)，NULL=不限
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(ColumnDef::new(Node::TrafficQuotaGb).double().null())
                    .to_owned(),
            )
            .await?;

        // 流量重置周期: none/daily/monthly
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(
                        ColumnDef::new(Node::TrafficResetCycle)
                            .string()
                            .default("none")
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 累计发送字节数
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(
                        ColumnDef::new(Node::TotalBytesSent)
                            .big_integer()
                            .default(0)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 累计接收字节数
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(
                        ColumnDef::new(Node::TotalBytesReceived)
                            .big_integer()
                            .default(0)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 上次重置时间
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(ColumnDef::new(Node::LastResetAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        // 流量超限标志
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(
                        ColumnDef::new(Node::IsTrafficExceeded)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 速度限制(字节/秒)，NULL=不限
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(ColumnDef::new(Node::SpeedLimit).big_integer().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            Node::MaxProxyCount,
            Node::AllowedPortRange,
            Node::TrafficQuotaGb,
            Node::TrafficResetCycle,
            Node::TotalBytesSent,
            Node::TotalBytesReceived,
            Node::LastResetAt,
            Node::IsTrafficExceeded,
            Node::SpeedLimit,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(Node::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Node {
    Table,
    MaxProxyCount,
    AllowedPortRange,
    TrafficQuotaGb,
    TrafficResetCycle,
    TotalBytesSent,
    TotalBytesReceived,
    LastResetAt,
    IsTrafficExceeded,
    SpeedLimit,
}
