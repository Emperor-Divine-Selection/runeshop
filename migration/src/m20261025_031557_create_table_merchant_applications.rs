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
                            "merchant_status",
                            ["pending", "active", "frozen", "rejected"],
                        )
                        .default("pending"),
                    )
                    .col(uuid("reviewed_by"))
                    .col(timestamp_with_time_zone("reviewed_at").default(Expr::current_timestamp()))
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
        manager
            .drop_table(Table::drop().table("merchant_applications").to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name("merchant_status").to_owned())
            .await
    }
}
