use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261121_130000_create_table_merchant_applications"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "../sql/m20261121_130000_create_table_merchant_applications.up.sql"
            ))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "../sql/m20261121_130000_create_table_merchant_applications.down.sql"
            ))
            .await?;
        Ok(())
    }
}
