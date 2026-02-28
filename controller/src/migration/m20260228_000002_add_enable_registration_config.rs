use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let insert = Query::insert()
            .into_table(SystemConfig::Table)
            .columns([
                SystemConfig::Key,
                SystemConfig::Value,
                SystemConfig::Description,
                SystemConfig::ValueType,
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
        let delete = Query::delete()
            .from_table(SystemConfig::Table)
            .and_where(Expr::col(SystemConfig::Key).eq("enable_registration"))
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
