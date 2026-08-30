use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260830_102617_create_spec_dims_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("spec_dims")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("product_id"))
                    .col(string("name").string_len(50).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-spec_dims-product_id")
                            .from("spec_dims", "product_id")
                            .to("products", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("spec_dims").to_owned())
            .await
    }
}
