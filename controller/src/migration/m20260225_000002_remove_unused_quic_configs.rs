use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除无用的 QUIC 配置项（这些配置实际上不会被 Node 使用）
        let delete = Query::delete()
            .from_table(SystemConfig::Table)
            .and_where(Expr::col(SystemConfig::Key).is_in([
                "health_check_interval",
                "idle_timeout",
                "keep_alive_interval",
                "max_concurrent_streams",
            ]))
            .to_owned();

        manager.exec_stmt(delete).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 恢复删除的配置项
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
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum SystemConfig {
    Table,
    Key,
    Value,
    Description,
    ValueType,
}
