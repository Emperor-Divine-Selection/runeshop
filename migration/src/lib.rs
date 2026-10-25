pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table_users;
mod m20260926_055846_alter_users_avatar_bio_nullable;
mod m20261015_160417_create_table_merchants;
mod m20261025_023122_create_table_backoffice_users;
mod m20261025_031557_create_table_merchant_applications;
mod m20261025_050000_normalize_user_identifiers;
mod m20261025_050001_add_uuid_defaults;
mod m20261025_050002_make_merchant_review_nullable;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table_users::Migration),
            Box::new(m20260926_055846_alter_users_avatar_bio_nullable::Migration),
            Box::new(m20261015_160417_create_table_merchants::Migration),
            Box::new(m20261025_023122_create_table_backoffice_users::Migration),
            Box::new(m20261025_031557_create_table_merchant_applications::Migration),
            Box::new(m20261025_050000_normalize_user_identifiers::Migration),
            Box::new(m20261025_050001_add_uuid_defaults::Migration),
            Box::new(m20261025_050002_make_merchant_review_nullable::Migration),
        ]
    }
}
