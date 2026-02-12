use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SystemConfig::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemConfig::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SystemConfig::Key)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(SystemConfig::Value).text().not_null())
                    .col(ColumnDef::new(SystemConfig::Description).string().not_null())
                    .col(ColumnDef::new(SystemConfig::ValueType).string().not_null())
                    .col(
                        ColumnDef::new(SystemConfig::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SystemConfig::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 插入默认配置
        let insert = Query::insert()
            .into_table(SystemConfig::Table)
            .columns([
                SystemConfig::Key,
                SystemConfig::Value,
                SystemConfig::Description,
                SystemConfig::ValueType,
            ])
            .values_panic([
                "health_check_interval".into(),
                "15".into(),
                "客户端连接健康检查间隔（秒）".into(),
                "number".into(),
            ])
            .values_panic([
                "idle_timeout".into(),
                "60".into(),
                "QUIC连接空闲超时时间（秒）".into(),
                "number".into(),
            ])
            .values_panic([
                "keep_alive_interval".into(),
                "5".into(),
                "QUIC Keep-Alive心跳间隔（秒）".into(),
                "number".into(),
            ])
            .values_panic([
                "max_concurrent_streams".into(),
                "100".into(),
                "最大并发流数量".into(),
                "number".into(),
            ])
            .values_panic([
                "system_name".into(),
                "\"RFRP\"".into(),
                "系统名称".into(),
                "string".into(),
            ])
            .values_panic([
                "enable_registration".into(),
                "false".into(),
                "是否开启用户注册".into(),
                "boolean".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SystemConfig::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SystemConfig {
    Table,
    Id,
    Key,
    Value,
    Description,
    ValueType,
    CreatedAt,
    UpdatedAt,
}
