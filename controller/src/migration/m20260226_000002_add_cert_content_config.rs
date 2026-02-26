use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加证书内容配置项（用于直接上传证书文件）
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
                        "grpc_tls_cert_content".into(),
                        "\"\"".into(),
                        "TLS certificate content (PEM format, base64 encoded)".into(),
                        "string".into(),
                    ])
                    .values_panic([
                        "grpc_tls_key_content".into(),
                        "\"\"".into(),
                        "TLS private key content (PEM format, base64 encoded)".into(),
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
                    .and_where(
                        Expr::col(SystemConfig::Key)
                            .is_in(["grpc_tls_cert_content", "grpc_tls_key_content"]),
                    )
                    .to_owned(),
            )
            .await?;

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
    CreatedAt,
    UpdatedAt,
}
