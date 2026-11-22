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
        // 店名和用户名一样：只在「待审」之间互斥，被拒后可以再申请
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE UNIQUE INDEX uq_pending_shop_name
                 ON merchant_applications (shop_name) WHERE status = 'pending'",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX uq_pending_shop_name")
            .await?;
        Ok(())
    }
}
