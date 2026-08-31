pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_users_table;
mod m20260827_161407_create_products_table;
mod m20260830_032511_create_orders_table;
mod m20260830_052511_create_orders_item_table;
mod m20260830_061001_add_avatar_to_users;
mod m20260830_074816_add_bio_in_users;
mod m20260830_102617_create_spec_dims_table;
mod m20260830_104216_create_spec_values_table;
mod m20260830_110840_create_product_variants_table;
mod m20260830_112446_create_variant_values_table;
mod m20260830_153457_create_wallents_table;
mod m20260830_154903_create_wallet_transactions_table;
mod m20260830_160749_add_xp_to_users;
mod m20260830_161223_create_xp_records_table;
mod m20260831_160833_create_member_levels_table;
mod m20260831_164256_create_user_memberships_table;
mod m20260831_170308_create_merchants_table;
mod m20260831_175558_create_merchant_accounts_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_users_table::Migration),
            Box::new(m20260827_161407_create_products_table::Migration),
            Box::new(m20260830_032511_create_orders_table::Migration),
            Box::new(m20260830_052511_create_orders_item_table::Migration),
            Box::new(m20260830_061001_add_avatar_to_users::Migration),
            Box::new(m20260830_074816_add_bio_in_users::Migration),
            Box::new(m20260830_102617_create_spec_dims_table::Migration),
            Box::new(m20260830_104216_create_spec_values_table::Migration),
            Box::new(m20260830_110840_create_product_variants_table::Migration),
            Box::new(m20260830_112446_create_variant_values_table::Migration),
            Box::new(m20260830_153457_create_wallents_table::Migration),
            Box::new(m20260830_154903_create_wallet_transactions_table::Migration),
            Box::new(m20260830_160749_add_xp_to_users::Migration),
            Box::new(m20260830_161223_create_xp_records_table::Migration),
            Box::new(m20260831_160833_create_member_levels_table::Migration),
            Box::new(m20260831_164256_create_user_memberships_table::Migration),
            Box::new(m20260831_170308_create_merchants_table::Migration),
            Box::new(m20260831_175558_create_merchant_accounts_table::Migration),
        ]
    }
}
