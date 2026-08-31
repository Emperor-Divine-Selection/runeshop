use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260831_create_merchants_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("merchants")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("user_id").unique_key()) // 1:1，一人一店
                    .col(string("shop_name").string_len(100))
                    .col(string("shop_logo").string_len(50))
                    .col(integer("contact_phone"))
                    .col(text("shop_description").null())
                    .col(string("status").string_len(20).default("pending"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-merchants-user_id")
                            .from("merchants", "user_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("merchants").to_owned())
            .await
    }
}
