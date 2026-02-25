use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加 gRPC TLS 配置到 SystemConfig 表
        let db = manager.get_connection();

        // 插入默认配置
        let insert_sql = r#"
            INSERT OR IGNORE INTO system_config (key, value, description, value_type, created_at, updated_at)
            VALUES
            ('grpc_tls_enabled', 'false', 'Enable TLS for gRPC server', 'boolean', datetime('now'), datetime('now')),
            ('grpc_tls_cert_path', '""', 'Path to TLS certificate file', 'string', datetime('now'), datetime('now')),
            ('grpc_tls_key_path', '""', 'Path to TLS private key file', 'string', datetime('now'), datetime('now')),
            ('grpc_domain', '""', 'gRPC server domain name (optional, for SNI)', 'string', datetime('now'), datetime('now'))
        "#;

        db.execute_unprepared(insert_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 删除 gRPC TLS 配置
        let delete_sql = r#"
            DELETE FROM system_config
            WHERE key IN ('grpc_tls_enabled', 'grpc_tls_cert_path', 'grpc_tls_key_path', 'grpc_domain')
        "#;

        db.execute_unprepared(delete_sql).await?;

        Ok(())
    }
}
