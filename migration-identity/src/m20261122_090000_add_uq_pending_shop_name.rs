use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261122_090000_add_uq_pending_shop_name"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "../sql/m20261122_090000_add_uq_pending_shop_name.up.sql"
            ))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "../sql/m20261122_090000_add_uq_pending_shop_name.down.sql"
            ))
            .await?;
        Ok(())
    }
}
