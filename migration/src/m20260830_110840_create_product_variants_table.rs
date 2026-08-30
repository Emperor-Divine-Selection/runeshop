use sea_orm_migration::{prelude::*, schema::*, sea_query::ForeignKeyAction::Cascade};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260830_110840_create_product_variants_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("product_variants")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("product_id"))
                    .col(string("sku_code").string_len(50))
                    .col(decimal("price").decimal_len(10, 2))
                    .col(integer("stock").default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-product_variants-product_id")
                            .from("product_variants", "product_id")
                            .to("products", "id")
                            .on_delete(Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("product_variants").to_owned())
            .await
    }
}
