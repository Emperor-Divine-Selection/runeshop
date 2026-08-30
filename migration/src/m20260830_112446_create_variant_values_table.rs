use sea_orm_migration::{prelude::*, schema::*, sea_query::ForeignKeyAction::Cascade};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260830_112446_create_variant_values_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("variant_values")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("variant_id"))
                    .col(integer("value_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-variant_values-variant_id")
                            .from("variant_values", "variant_id")
                            .to("product_variants", "id")
                            .on_delete(Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-variant_values-value_id")
                            .from("variant_values", "value_id")
                            .to("spec_values", "id")
                            .on_delete(Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("variant_values").to_owned())
            .await
    }
}
