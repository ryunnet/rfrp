use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 client 表
        manager
            .create_table(
                Table::create()
                    .table(Client::Table)
                    .if_not_exists()
                    .col(big_integer(Client::Id).auto_increment().primary_key())
                    .col(string(Client::Name))
                    .col(string(Client::Token))
                    .col(boolean(Client::IsOnline).default(false))
                    .col(big_integer(Client::TotalBytesSent).default(0))
                    .col(big_integer(Client::TotalBytesReceived).default(0))
                    .col(timestamp(Client::CreatedAt))
                    .col(timestamp(Client::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 为 token 创建唯一索引
        manager
            .create_index(
                Index::create()
                    .name("idx_client_token")
                    .table(Client::Table)
                    .col(Client::Token)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 创建 user 表
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(big_integer(User::Id).auto_increment().primary_key())
                    .col(string(User::Username).unique_key())
                    .col(string(User::PasswordHash))
                    .col(boolean(User::IsAdmin).default(false))
                    .col(big_integer(User::TotalBytesSent).default(0))
                    .col(big_integer(User::TotalBytesReceived).default(0))
                    .col(timestamp(User::CreatedAt))
                    .col(timestamp(User::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 为 username 创建唯一索引
        manager
            .create_index(
                Index::create()
                    .name("idx_user_username")
                    .table(User::Table)
                    .col(User::Username)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 创建 proxy 表
        manager
            .create_table(
                Table::create()
                    .table(Proxy::Table)
                    .if_not_exists()
                    .col(big_integer(Proxy::Id).auto_increment().primary_key())
                    .col(string(Proxy::Name))
                    .col(string(Proxy::ProxyType))
                    .col(string(Proxy::LocalIp))
                    .col(integer(Proxy::LocalPort))
                    .col(integer(Proxy::RemotePort))
                    .col(boolean(Proxy::Enabled))
                    .col(big_integer(Proxy::TotalBytesSent).default(0))
                    .col(big_integer(Proxy::TotalBytesReceived).default(0))
                    .col(timestamp(Proxy::CreatedAt))
                    .col(timestamp(Proxy::UpdatedAt))
                    .col(string(Proxy::ClientId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_proxy_client")
                            .from(Proxy::Table, Proxy::ClientId)
                            .to(Client::Table, Client::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 proxy 索引
        manager
            .create_index(
                Index::create()
                    .name("idx_proxy_client_id")
                    .table(Proxy::Table)
                    .col(Proxy::ClientId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_proxy_remote_port")
                    .table(Proxy::Table)
                    .col(Proxy::RemotePort)
                    .to_owned(),
            )
            .await?;

        // 创建 user_client 表 (用户-客户端关联表)
        manager
            .create_table(
                Table::create()
                    .table(UserClient::Table)
                    .if_not_exists()
                    .col(big_integer(UserClient::Id).auto_increment().primary_key())
                    .col(big_integer(UserClient::UserId))
                    .col(big_integer(UserClient::ClientId))
                    .col(timestamp(UserClient::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_client_user")
                            .from(UserClient::Table, UserClient::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_client_client")
                            .from(UserClient::Table, UserClient::ClientId)
                            .to(Client::Table, Client::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建唯一索引 (user_id, client_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_user_client_unique")
                    .table(UserClient::Table)
                    .col(UserClient::UserId)
                    .col(UserClient::ClientId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 创建 traffic_daily 表
        manager
            .create_table(
                Table::create()
                    .table(TrafficDaily::Table)
                    .if_not_exists()
                    .col(big_integer(TrafficDaily::Id).auto_increment().primary_key())
                    .col(big_integer(TrafficDaily::ProxyId))
                    .col(big_integer(TrafficDaily::ClientId))
                    .col(big_integer(TrafficDaily::BytesSent).default(0))
                    .col(big_integer(TrafficDaily::BytesReceived).default(0))
                    .col(string(TrafficDaily::Date))
                    .col(timestamp(TrafficDaily::CreatedAt))
                    .col(timestamp(TrafficDaily::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_traffic_daily_proxy")
                            .from(TrafficDaily::Table, TrafficDaily::ProxyId)
                            .to(Proxy::Table, Proxy::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_traffic_daily_client")
                            .from(TrafficDaily::Table, TrafficDaily::ClientId)
                            .to(Client::Table, Client::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建唯一索引 (proxy_id, date)
        manager
            .create_index(
                Index::create()
                    .name("idx_traffic_daily_proxy_date")
                    .table(TrafficDaily::Table)
                    .col(TrafficDaily::ProxyId)
                    .col(TrafficDaily::Date)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 创建索引 (client_id, date) 用于查询
        manager
            .create_index(
                Index::create()
                    .name("idx_traffic_daily_client_date")
                    .table(TrafficDaily::Table)
                    .col(TrafficDaily::ClientId)
                    .col(TrafficDaily::Date)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrafficDaily::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(UserClient::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Proxy::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Client::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Client {
    Table,
    Id,
    Name,
    Token,
    IsOnline,
    TotalBytesSent,
    TotalBytesReceived,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Username,
    PasswordHash,
    IsAdmin,
    TotalBytesSent,
    TotalBytesReceived,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Proxy {
    Table,
    Id,
    ClientId,
    Name,
    ProxyType,
    LocalIp,
    LocalPort,
    RemotePort,
    Enabled,
    TotalBytesSent,
    TotalBytesReceived,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserClient {
    Table,
    Id,
    UserId,
    ClientId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum TrafficDaily {
    Table,
    Id,
    ProxyId,
    ClientId,
    BytesSent,
    BytesReceived,
    Date,
    CreatedAt,
    UpdatedAt,
}
