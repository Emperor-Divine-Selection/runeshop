use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261025_050002_make_merchant_review_nullable"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("merchant_applications"))
                    .modify_column(ColumnDef::new(Alias::new("reviewed_by")).uuid().null())
                    .modify_column(
                        ColumnDef::new(Alias::new("reviewed_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE merchant_applications ALTER COLUMN reviewed_at DROP DEFAULT",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("merchant_applications"))
                    .modify_column(ColumnDef::new(Alias::new("reviewed_by")).uuid().not_null())
                    .modify_column(
                        ColumnDef::new(Alias::new("reviewed_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }
}
