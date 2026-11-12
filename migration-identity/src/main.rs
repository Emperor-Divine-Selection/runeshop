use std::env;

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Database, DbBackend, DbErr, Statement};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let maintenance_url = env::var("POSTGRES_URL_MAINTENANCE")?;
    let identity_url = env::var("POSTGRES_URL_IDENTITY")?;

    ensure_database(&maintenance_url, "identity_db").await?;

    let db = Database::connect(&identity_url).await?;
    migration_identity::Migrator::up(&db, None).await?;

    Ok(())
}

/// 连维护库，缺库才建。名字来自常量，不是用户输入。
async fn ensure_database(maintenance_url: &str, database_name: &str) -> Result<(), DbErr> {
    let admin = Database::connect(maintenance_url).await?;

    let exists = admin
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT 1 FROM pg_database WHERE datname = $1",
            [database_name.into()],
        ))
        .await?;

    if exists.is_some() {
        return Ok(());
    }

    admin
        .execute_unprepared(&format!(r#"CREATE DATABASE "{database_name}""#))
        .await?;

    Ok(())
}
