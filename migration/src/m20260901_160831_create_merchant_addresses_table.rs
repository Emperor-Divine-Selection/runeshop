use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260901_160831_create_merchant_addresses_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("merchant_addresses")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("merchant_id").unique_key()) // 1:1，一店一址
                    .col(string("province").string_len(50))
                    .col(string("city").string_len(50))
                    .col(string("district").string_len(50))
                    .col(string("detail").string_len(255))
                    .col(string("contact_name").string_len(50))
                    .col(string("contact_phone").string_len(20))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-merchant_addresses-merchant_id")
                            .from("merchant_addresses", "merchant_id")
                            .to("merchants", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("merchant_addresses").to_owned())
            .await
    }
}
