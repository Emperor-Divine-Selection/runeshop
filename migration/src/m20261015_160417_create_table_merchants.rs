use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261015_160417_create_table_merchants"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(("merchant", "merchant_status"))
                    .values(["active", "frozen"])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(("merchant", "merchants"))
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("gen_random_uuid()")))
                    .col(string("username").not_null().unique_key())
                    .col(string("password_hash").not_null())
                    .col(string("email").not_null().unique_key())
                    .col(
                        timestamp_with_time_zone("create_time")
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone("update_time")
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(string("shop_name").not_null())
                    .col(
                        // ColumnType::Custom 是原样输出的，所以可以直接写 schema.type。
                        // 不能用 enumeration()：它只接单个标识符，而 PG 建表时
                        // 不会用 search_path 去别的 schema 找类型
                        ColumnDef::new(Alias::new("status"))
                            .custom("merchant.merchant_status")
                            .default("active")
                            .not_null()
                            .take(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(("merchant", "merchants")).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(("merchant", "merchant_status")).to_owned())
            .await
    }
}
