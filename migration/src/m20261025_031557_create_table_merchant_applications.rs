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
        // 申请单有自己的状态机：pending（待审）→ approved / rejected（终态）
        // 不能借用 merchants 的 merchant_status（active / frozen），那是营业资格，会变
        manager
            .create_type(
                Type::create()
                    .as_enum("application_status")
                    .values(["pending", "approved", "rejected"])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("merchant_applications")
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(uuid("user_id").not_null().unique_key())
                    .col(timestamp_with_time_zone("create_time").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("update_time").default(Expr::current_timestamp()))
                    .col(string("shop_name").not_null().unique_key())
                    .col(
                        enumeration(
                            "status",
                            "application_status",
                            ["pending", "approved", "rejected"],
                        )
                        .default("pending")
                        .not_null(),
                    )
                    // 审核人 / 审核时间都由「审核动作」写入，所以可空、无默认值
                    // （sea-query 的列默认是 NOT NULL，要可空必须显式 .null()，见踩坑 26）
                    // （有默认值的话，未审核的行也会像审核过，见踩坑 97）
                    .col(uuid("reviewed_by").null())
                    .col(timestamp_with_time_zone("reviewed_at").null())
                    // 非 pending 必须有审核人 + 审核时间，把「改了一半」挡在数据库层
                    .check(Check::named(
                        "chk_merchant_applications_review_consistency",
                        Expr::col(Alias::new("status"))
                            .eq("pending")
                            .and(Expr::col(Alias::new("reviewed_at")).is_null())
                            .and(Expr::col(Alias::new("reviewed_by")).is_null())
                            .or(Expr::col(Alias::new("status"))
                                .ne("pending")
                                .and(Expr::col(Alias::new("reviewed_at")).is_not_null())
                                .and(Expr::col(Alias::new("reviewed_by")).is_not_null())),
                    ))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_merchant_applications_user_id")
                            .from("merchant_applications", "user_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_merchant_applications_reviewed_by")
                            .from("merchant_applications", "reviewed_by")
                            .to("backoffice_users", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 顺序不能反：先删表（否则类型还被列引用着），再删类型
        manager
            .drop_table(Table::drop().table("merchant_applications").to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name("application_status").to_owned())
            .await
    }
}
