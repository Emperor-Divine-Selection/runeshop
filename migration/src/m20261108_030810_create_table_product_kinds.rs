use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261108_030810_create_table_product_kinds"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(("catalog", "product_kinds"))
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("gen_random_uuid()")))
                    .col(uuid("product_id").not_null())
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
                    .index(
                        Index::create()
                            .unique()
                            .name("uk_product_kinds_product_id_name")
                            .col("product_id")
                            .col("name"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_kinds_product_id")
                            .from(("catalog", "product_kinds"), "product_id")
                            .to(("catalog", "products"), "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(("catalog", "product_kinds")).to_owned())
            .await
    }
}
