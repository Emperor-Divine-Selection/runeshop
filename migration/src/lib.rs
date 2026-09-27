pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table_users;
mod m20260926_055846_alter_users_avatar_bio_nullable;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table_users::Migration),
            Box::new(m20260926_055846_alter_users_avatar_bio_nullable::Migration),
        ]
    }
}
