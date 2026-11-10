use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261108_041855_create_table_product_inventory"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(("catalog", "product_inventory"))
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("gen_random_uuid()")))
                    .col(uuid("item_id").not_null())
                    .col(
                        timestamp_with_time_zone("create_time")
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone("update_time")
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(integer("stock").not_null().default(0))
                    .index(
                        Index::create()
                            .unique()
                            .name("uk_product_inventory_item_id")
                            .col("item_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_inventory_item_id")
                            .from(("catalog", "product_inventory"), "item_id")
                            .to(("catalog", "product_items"), "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .check(Check::named(
                        "chk_product_inventory_stock_non_negative",
                        Expr::col(Alias::new("stock")).gte(0),
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(("catalog", "product_inventory")).to_owned())
            .await
    }
}
