use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261025_023122_create_table_backoffice_users"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("backoffice_users")
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
            .drop_table(Table::drop().table("backoffice_users").to_owned())
            .await
    }
}
