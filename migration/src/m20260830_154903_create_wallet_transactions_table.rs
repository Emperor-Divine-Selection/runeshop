use sea_orm_migration::{prelude::*, schema::*, sea_query::ForeignKeyAction::Cascade};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260830_154903_create_wallet_transactions_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("wallet_transactions")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("wallet_id"))
                    .col(decimal("amount").decimal_len(10, 2)) //带符号金额
                    .col(string("tx_type").string_len(255).null()) //交易类型
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp())) //备注
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-wallet_transactions-wallet_id")
                            .from("wallet_transactions", "wallet_id")
                            .to("wallets", "id")
                            .on_delete(Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("wallet_transactions").to_owned())
            .await
    }
}
