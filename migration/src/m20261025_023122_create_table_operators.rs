use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261025_023122_create_table_operators"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 运营人员独立表：不与顾客共用 users，也不与商户共用 merchants
        // 表名按「人是谁」命名（operators = 运营），不按「从哪进来」命名（backoffice 是入口）
        manager
            .create_table(
                Table::create()
                    .table(("identity", "operators"))
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(timestamp_with_time_zone("create_time").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("update_time").default(Expr::current_timestamp()))
                    .col(string("username").not_null().unique_key())
                    .col(string("password_hash").not_null())
                    .col(string("email").unique_key())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(("identity", "operators")).to_owned())
            .await
    }
}
