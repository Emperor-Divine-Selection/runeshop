use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;
use sea_orm_migration::sea_orm::sea_query::extension::postgres::Type;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261121_130000_create_table_merchant_applications"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 状态用 PG 原生 enum：类型本身就拒绝非法值，不靠 CHECK 兜底
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
                    .table(MerchantApplications::Table)
                    .col(pk_uuid(MerchantApplications::Id))
                    .col(timestamp_with_time_zone_default_now(
                        MerchantApplications::CreatedAt,
                    ))
                    .col(timestamp_with_time_zone_default_now(
                        MerchantApplications::UpdatedAt,
                    ))
                    .col(string_len(MerchantApplications::ShopName, 64))
                    .col(string_len(MerchantApplications::Username, 32))
                    .col(string_len(MerchantApplications::Email, 255))
                    .col(
                        ColumnDef::new(MerchantApplications::Status)
                            .custom("application_status")
                            .not_null()
                            .default("pending"),
                    )
                    // 审核痕迹：可空、无默认（踩坑 97）
                    .col(uuid_null(MerchantApplications::ReviewedBy))
                    .col(timestamp_with_time_zone_null(
                        MerchantApplications::ReviewedAt,
                    ))
                    .col(text_null(MerchantApplications::ReviewNote))
                    // 挡「改了一半」的更新（踩坑 103）
                    .check(Check::named(
                        "chk_review_consistency",
                        Condition::any()
                            .add(
                                Condition::all()
                                    .add(Expr::col(MerchantApplications::Status).eq("pending"))
                                    .add(Expr::col(MerchantApplications::ReviewedBy).is_null())
                                    .add(Expr::col(MerchantApplications::ReviewedAt).is_null()),
                            )
                            .add(
                                Condition::all()
                                    .add(Expr::col(MerchantApplications::Status).ne("pending"))
                                    .add(Expr::col(MerchantApplications::ReviewedBy).is_not_null())
                                    .add(Expr::col(MerchantApplications::ReviewedAt).is_not_null()),
                            ),
                    ))
                    .to_owned(),
            )
            .await?;

        // 同一个商户名 / 邮箱，同时只能有一条待审申请
        manager
            .create_index(
                Index::create()
                    .name("uq_pending_username")
                    .table(MerchantApplications::Table)
                    .col(MerchantApplications::Username)
                    .unique()
                    .and_where(Expr::col(MerchantApplications::Status).eq("pending"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_pending_email")
                    .table(MerchantApplications::Table)
                    .col(MerchantApplications::Email)
                    .unique()
                    .and_where(Expr::col(MerchantApplications::Status).eq("pending"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MerchantApplications::Table).to_owned())
            .await?;

        // 类型被列引用着，必须等表删完再删
        manager
            .drop_type(Type::drop().name("application_status").to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MerchantApplications {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    ShopName,
    Username,
    Email,
    Status,
    ReviewedBy,
    ReviewedAt,
    ReviewNote,
}
