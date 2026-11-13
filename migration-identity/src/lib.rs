pub use sea_orm_migration::prelude::*;

pub struct Migrator;

mod m20261113_140503_create_table_customers;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20261113_140503_create_table_customers::Migration)]
    }
}
