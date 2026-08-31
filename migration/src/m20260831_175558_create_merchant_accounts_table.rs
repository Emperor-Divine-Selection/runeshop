use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260831_create_merchant_accounts_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("merchant_accounts")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("merchant_id").unique_key()) // 一店一账户
                    .col(string("account_name").string_len(100))
                    .col(string("account_number").string_len(50)) // 生产环境存加密值
                    .col(string("bank_name").string_len(100))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-merchant_accounts-merchant_id")
                            .from("merchant_accounts", "merchant_id")
                            .to("merchants", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("merchant_accounts").to_owned())
            .await
    }
}
