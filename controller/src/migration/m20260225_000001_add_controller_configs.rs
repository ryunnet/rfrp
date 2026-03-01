use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加 Controller 相关的配置项
        let insert = Query::insert()
            .into_table(SystemConfig::Table)
            .columns([
                SystemConfig::Key,
                SystemConfig::Value,
                SystemConfig::Description,
                SystemConfig::ValueType,
            ])
            .values_panic([
                "web_port".into(),
                "3000".into(),
                "Web 管理界面端口".into(),
                "number".into(),
            ])
            .values_panic([
                "internal_port".into(),
                "3100".into(),
                "gRPC 服务端口（Node 和 Client 连接使用）".into(),
                "number".into(),
            ])
            .values_panic([
                "jwt_expiration_hours".into(),
                "24".into(),
                "JWT 过期时间（小时）".into(),
                "number".into(),
            ])
            .values_panic([
                "db_path".into(),
                "\"./data/oxiproxy.db\"".into(),
                "数据库路径".into(),
                "string".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除添加的配置项
        let delete = Query::delete()
            .from_table(SystemConfig::Table)
            .and_where(Expr::col(SystemConfig::Key).is_in([
                "web_port",
                "internal_port",
                "jwt_expiration_hours",
                "db_path",
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
