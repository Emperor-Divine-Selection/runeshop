use sea_orm_migration::{prelude::*, schema::*, sea_query::ForeignKeyAction::Cascade};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260831_164256_create_user_memberships_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("user_memberships")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("user_id"))
                    .col(integer("level_id"))
                    .col(timestamp_with_time_zone("granted_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_memberships-user_id")
                            .from("user_memberships", "user_id")
                            .to("users", "id")
                            .on_delete(Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("user_memberships").to_owned())
            .await
    }
}
