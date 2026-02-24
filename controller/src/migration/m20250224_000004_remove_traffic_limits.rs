use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除 user 表的 upload_limit_gb 和 download_limit_gb 字段
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

        // 删除 client 表的 upload_limit_gb 和 download_limit_gb 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .drop_column(Client::UploadLimitGb)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .drop_column(Client::DownloadLimitGb)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚：重新添加 user 表的字段
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::UploadLimitGb)
                            .double()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::DownloadLimitGb)
                            .double()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        // 回滚：重新添加 client 表的字段
        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .add_column(
                        ColumnDef::new(Client::UploadLimitGb)
                            .double()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Client::Table)
                    .add_column(
                        ColumnDef::new(Client::DownloadLimitGb)
                            .double()
                            .null()
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    UploadLimitGb,
    DownloadLimitGb,
}

#[derive(DeriveIden)]
enum Client {
    Table,
    UploadLimitGb,
    DownloadLimitGb,
}
