use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260830_052511_create_orders_item_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("order_items")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("order_id"))
                    .col(integer("product_id"))
                    .col(integer("quantity"))
                    .col(decimal("price").decimal_len(10, 2))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_items-order_id")
                            .from("order_items", "order_id")
                            .to("orders", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_items-product_id")
                            .from("order_items", "product_id")
                            .to("products", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("order_items").to_owned())
            .await
    }
}
