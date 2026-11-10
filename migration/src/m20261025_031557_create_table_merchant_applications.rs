use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261025_031557_create_table_merchant_applications"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 申请单有自己的状态机：pending（待审）→ approved / rejected / supplement_required
        // 不能借用 merchants 的 merchant_status（active / frozen），那是营业资格，会变
        // supplement_required 是「可回退」的：申请人补完资料后回到 pending
        manager
            .create_type(
                Type::create()
                    .as_enum(("merchant", "application_status"))
                    .values(["pending", "approved", "rejected", "supplement_required"])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(("merchant", "merchant_applications"))
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(string("username").not_null())
                    .col(string("password_hash").not_null())
                    .col(string("email").not_null())
                    .col(timestamp_with_time_zone("create_time").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("update_time").default(Expr::current_timestamp()))
                    .col(string("shop_name").not_null().unique_key())
                    .col(
                        // 列类型写全限定名（ColumnType::Custom 原样输出）——
                        // PG 建表时不会用 search_path 去别的 schema 找类型
                        ColumnDef::new(Alias::new("status"))
                            .custom("merchant.application_status")
                            .default("pending")
                            .not_null()
                            .take(),
                    )
                    // 审核人 / 审核时间都由「审核动作」写入，所以可空、无默认值
                    // （sea-query 的列默认是 NOT NULL，要可空必须显式 .null()，见踩坑 26）
                    // （有默认值的话，未审核的行也会像审核过，见踩坑 97）
                    .col(uuid("reviewed_by").null())
                    .col(timestamp_with_time_zone("reviewed_at").null())
                    // 只有 pending 允许没有审核人；其余三个状态必须有人有时间。
                    // pending 允许「有」审核记录 —— 那是「被打回过一次」的痕迹
                    .check(Check::named(
                        "chk_merchant_applications_review_consistency",
                        Expr::cust(
                            "status = 'pending' \
                             OR (reviewed_by IS NOT NULL AND reviewed_at IS NOT NULL)",
                        ),
                    ))
                    .to_owned(),
            )
            .await?;

        // 同一个用户名同时只能有一条「在流程中」的申请（待审 + 待补正）。
        // 被驳回 / 通过之后不再占用这个用户名，所以用部分唯一索引而不是普通唯一约束
        manager
            .create_index(
                Index::create()
                    .name("uq_active_app_username")
                    .table(("merchant", "merchant_applications"))
                    .col("username")
                    .unique()
                    .and_where(Expr::col(Alias::new("status")).is_in([
                        "pending",
                        "supplement_required",
                    ]))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 顺序不能反：先删表（否则类型还被列引用着），再删类型
        manager
            .drop_table(Table::drop().table(("merchant", "merchant_applications")).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(("merchant", "application_status")).to_owned())
            .await
    }
}
