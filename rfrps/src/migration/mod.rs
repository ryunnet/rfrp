use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use std::fs::create_dir_all;
use std::{fs, path};
use tokio::sync::OnceCell;

mod m20250119_000001_init;
mod m20250126_000001_add_traffic_limit;
mod m20250126_000002_create_system_config;
mod m20250128_000001_add_client_traffic_limit;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250119_000001_init::Migration),
            Box::new(m20250126_000001_add_traffic_limit::Migration),
            Box::new(m20250126_000002_create_system_config::Migration),
            Box::new(m20250128_000001_add_client_traffic_limit::Migration),
        ]
    }
}

static DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_connection() -> &'static DatabaseConnection {
    DATABASE_CONNECTION.get_or_init(init_sqlite).await
}

pub async fn init_sqlite() -> DatabaseConnection {
    let path = path::Path::new("data/rfrps.db");
    if !path.exists() {
        if let Some(parent) = path.parent() {
            create_dir_all(parent).unwrap();
        }
        fs::write(path, "").unwrap();
    }
    let db = Database::connect("sqlite://data/rfrps.db")
        .await
        .expect("failed to connect sqlite");

    db
}
