use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 插入协议选择配置项
        let insert = Query::insert()
            .into_table(SystemConfig::Table)
            .columns([
                SystemConfig::Key,
                SystemConfig::Value,
                SystemConfig::Description,
                SystemConfig::ValueType,
            ])
            .values_panic([
                "use_kcp".into(),
                "false".into(),
                "使用KCP协议（否则使用QUIC）".into(),
                "boolean".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除协议选择配置项
        let delete = Query::delete()
            .from_table(SystemConfig::Table)
            .and_where(Expr::col(SystemConfig::Key).eq("use_kcp"))
            .to_owned();

        manager.exec_stmt(delete).await?;

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
