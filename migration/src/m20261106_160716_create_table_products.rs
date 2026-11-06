use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261106_160716_create_table_products"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 建枚举：enumeration() 只是引用类型，不创建
        // product_items 也用这个类型，但它的迁移排在本条之后，
        // 回滚时顺序相反（items 先 down），所以在这里删类型是安全的
        manager
            .create_type(
                Type::create()
                    .as_enum("product_status")
                    .values(["on_sale", "off_shelf"])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("products")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("gen_random_uuid()")))
                    .col(uuid("merchant_id").not_null())
                    .col(string("name").not_null())
                    // 描述可能很长，用 text 而不是 varchar
                    .col(text("description").null())
                    // 只存对象 key，不存完整 URL（和 users.avatar 一致）
                    .col(string("cover_image").null())
                    .col(
                        enumeration("status", "product_status", ["on_sale", "off_shelf"])
                            .default("on_sale")
                            .not_null(),
                    )
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
                    // 商品属于店铺，不属于人：商户被删，商品跟着走
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_products_merchant_id")
                            .from("products", "merchant_id")
                            .to("merchants", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // PG 的外键不会自动建索引，按商户查商品会全表扫
        manager
            .create_index(
                Index::create()
                    .name("idx_products_merchant_id")
                    .table("products")
                    .col("merchant_id")
                    .to_owned(),
            )
            .await?;

        // 同一店铺不能有两个同名商品
        manager
            .create_index(
                Index::create()
                    .name("uk_products_merchant_id_name")
                    .table("products")
                    .col("merchant_id")
                    .col("name")
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 顺序不能反：先删表（索引随表走），再删类型
        manager
            .drop_table(Table::drop().table("products").to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name("product_status").to_owned())
            .await
    }
}
