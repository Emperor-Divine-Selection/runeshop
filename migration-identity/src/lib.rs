pub use sea_orm_migration::prelude::*;

pub struct Migrator;

mod m20261113_140503_create_table_customers;
mod m20261121_130000_create_table_merchant_applications;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261113_140503_create_table_customers::Migration),
            Box::new(m20261121_130000_create_table_merchant_applications::Migration),
        ]
    }
}
