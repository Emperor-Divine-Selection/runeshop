use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261108_034028_create_table_product_items"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("product_items")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("gen_random_uuid()")))
                    .col(uuid("kind_id").not_null())
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
                    .col(string("name").not_null())
                    .col(integer("sort_order").not_null().default(0))
                    .col(
                        enumeration("status", "product_status", ["on_sale", "off_shelf"])
                            .default("on_sale")
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("uk_product_items_kind_id_name")
                            .col("kind_id")
                            .col("name"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_items_kind_id")
                            .from("product_items", "kind_id")
                            .to("product_kinds", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("product_items").to_owned())
            .await
    }
}
