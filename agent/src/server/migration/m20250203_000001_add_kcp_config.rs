use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 插入 KCP 配置项（不包含 kcp_port，因为现在使用统一端口）
        let insert = Query::insert()
            .into_table(SystemConfig::Table)
            .columns([
                SystemConfig::Key,
                SystemConfig::Value,
                SystemConfig::Description,
                SystemConfig::ValueType,
            ])
            .values_panic([
                "kcp_nodelay".into(),
                "true".into(),
                "KCP无延迟模式：启用后禁用Nagle算法，降低延迟".into(),
                "boolean".into(),
            ])
            .values_panic([
                "kcp_interval".into(),
                "10".into(),
                "KCP内部更新时钟间隔（毫秒），值越小延迟越低".into(),
                "number".into(),
            ])
            .values_panic([
                "kcp_resend".into(),
                "2".into(),
                "KCP快速重传触发次数，0表示禁用快速重传".into(),
                "number".into(),
            ])
            .values_panic([
                "kcp_nc".into(),
                "true".into(),
                "KCP关闭拥塞控制：启用后发送速度更快".into(),
                "boolean".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除 KCP 配置项
        let delete = Query::delete()
            .from_table(SystemConfig::Table)
            .and_where(Expr::col(SystemConfig::Key).is_in([
                "kcp_nodelay",
                "kcp_interval",
                "kcp_resend",
                "kcp_nc",
            ]))
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
