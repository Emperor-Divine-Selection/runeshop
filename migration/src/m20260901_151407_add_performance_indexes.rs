use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260831_add_performance_indexes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 给"经常查询/JOIN"的列逐个建索引
        manager
            .create_index(
                Index::create()
                    .name("idx-orders-user_id")
                    .table("orders")
                    .col(Alias::new("user_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-order_items-order_id")
                    .table("order_items")
                    .col(Alias::new("order_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-order_items-product_id")
                    .table("order_items")
                    .col(Alias::new("product_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-xp_records-user_id")
                    .table("xp_records")
                    .col(Alias::new("user_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-wallet_transactions-wallet_id")
                    .table("wallet_transactions")
                    .col(Alias::new("wallet_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-spec_values-dim_id")
                    .table("spec_values")
                    .col(Alias::new("dim_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-product_variants-product_id")
                    .table("product_variants")
                    .col(Alias::new("product_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-variant_values-variant_id")
                    .table("variant_values")
                    .col(Alias::new("variant_id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 对称：回滚时逐个删索引
        manager
            .drop_index(Index::drop().name("idx-orders-user_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx-order_items-order_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx-order_items-product_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx-xp_records-user_id").to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-wallet_transactions-wallet_id")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(Index::drop().name("idx-spec_values-dim_id").to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-product_variants-product_id")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-variant_values-variant_id")
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
