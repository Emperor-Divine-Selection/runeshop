use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261113_140503_create_table_customers"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Customers::Table)
                    .col(pk_uuid(Customers::Id))
                    .col(timestamp_with_time_zone_default_now(Customers::CreatedAt))
                    .col(timestamp_with_time_zone_default_now(Customers::UpdatedAt))
                    .col(string_len_uniq(Customers::Username, 32))
                    .col(string_len(Customers::PasswordHash, 255))
                    .col(string_len_uniq(Customers::Email, 255))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Customers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    CreatedAt,
    UpdatedAt,
}
