use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260831_160833_create_member_levels_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("member_levels")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(string("name").string_len(50))
                    .col(decimal("discount").decimal_len(3, 2))
                    .col(integer("min_xp"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("member_levels").to_owned())
            .await
    }
}
