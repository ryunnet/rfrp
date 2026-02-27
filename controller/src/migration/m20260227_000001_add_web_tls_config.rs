use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加 Web TLS 配置项
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(SystemConfig::Table)
                    .columns([
                        SystemConfig::Key,
                        SystemConfig::Value,
                        SystemConfig::Description,
                        SystemConfig::ValueType,
                    ])
                    .values_panic([
                        "web_tls_enabled".into(),
                        "\"false\"".into(),
                        "是否启用 Web 管理界面 TLS".into(),
                        "boolean".into(),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(SystemConfig::Table)
                    .columns([
                        SystemConfig::Key,
                        SystemConfig::Value,
                        SystemConfig::Description,
                        SystemConfig::ValueType,
                    ])
                    .values_panic([
                        "web_tls_cert_path".into(),
                        "\"\"".into(),
                        "Web TLS 证书文件路径".into(),
                        "string".into(),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(SystemConfig::Table)
                    .columns([
                        SystemConfig::Key,
                        SystemConfig::Value,
                        SystemConfig::Description,
                        SystemConfig::ValueType,
                    ])
                    .values_panic([
                        "web_tls_key_path".into(),
                        "\"\"".into(),
                        "Web TLS 私钥文件路径".into(),
                        "string".into(),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(SystemConfig::Table)
                    .columns([
                        SystemConfig::Key,
                        SystemConfig::Value,
                        SystemConfig::Description,
                        SystemConfig::ValueType,
                    ])
                    .values_panic([
                        "web_tls_cert_content".into(),
                        "\"\"".into(),
                        "Web TLS 证书内容（Base64）".into(),
                        "string".into(),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(SystemConfig::Table)
                    .columns([
                        SystemConfig::Key,
                        SystemConfig::Value,
                        SystemConfig::Description,
                        SystemConfig::ValueType,
                    ])
                    .values_panic([
                        "web_tls_key_content".into(),
                        "\"\"".into(),
                        "Web TLS 私钥内容（Base64）".into(),
                        "string".into(),
                    ])
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(SystemConfig::Table)
                    .and_where(Expr::col(SystemConfig::Key).is_in([
                        "web_tls_enabled",
                        "web_tls_cert_path",
                        "web_tls_key_path",
                        "web_tls_cert_content",
                        "web_tls_key_content",
                    ]))
                    .to_owned(),
            )
            .await
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
