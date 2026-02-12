use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 为 user 表添加流量限制相关字段
        // SQLite 不支持单个 ALTER TABLE 中多个操作，需要分别执行

        // 上传流量限制（GB），NULL 表示无限制
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(ColumnDef::new(User::UploadLimitGb).double().null())
                    .to_owned(),
            )
            .await?;

        // 下载流量限制（GB），NULL 表示无限制
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(ColumnDef::new(User::DownloadLimitGb).double().null())
                    .to_owned(),
            )
            .await?;

        // 流量重置周期：'none', 'daily', 'monthly'
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::TrafficResetCycle)
                            .string()
                            .default("none")
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 上次重置时间
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::LastResetAt)
                            .timestamp()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 是否已超过流量限制
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::IsTrafficExceeded)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚：删除添加的字段
        // SQLite 不支持单个 ALTER TABLE 中多个操作，需要分别执行

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::UploadLimitGb)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::DownloadLimitGb)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::TrafficResetCycle)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::LastResetAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::IsTrafficExceeded)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    UploadLimitGb,
    DownloadLimitGb,
    TrafficResetCycle,
    LastResetAt,
    IsTrafficExceeded,
}
