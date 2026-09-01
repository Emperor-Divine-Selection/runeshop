use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260901_152245_create_table_user_addresses"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("user_addresses")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("user_id"))
                    .col(string("recipient").string_len(50))
                    .col(string("phone").string_len(20))
                    .col(string("province").string_len(50))
                    .col(string("city").string_len(50))
                    .col(string("district").string_len(50))
                    .col(string("detail").string_len(255))
                    .col(boolean("is_default").default(false))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_addresses-users")
                            .from("user_addresses", "user_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("user_addresses").to_owned())
            .await
    }
}
