use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let keys = [
            "system_name",
            "enable_registration",
            "kcp_nodelay",
            "kcp_interval",
            "kcp_resend",
            "kcp_nc",
            "use_kcp",
        ];

        for key in keys {
            let delete = Query::delete()
                .from_table(SystemConfig::Table)
                .and_where(Expr::col(SystemConfig::Key).eq(key))
                .to_owned();
            manager.exec_stmt(delete).await?;
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum SystemConfig {
    Table,
    Key,
}
