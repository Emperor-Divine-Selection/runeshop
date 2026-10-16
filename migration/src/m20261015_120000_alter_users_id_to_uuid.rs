use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261015_120000_alter_users_id_to_uuid"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("users")
                    .add_column(
                        ColumnDef::new("id_uuid")
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(Table::alter().table("users").drop_column("id").to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("users")
                    .rename_column("id_uuid", "id")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("users")
                    .modify_column(ColumnDef::new("id").primary_key())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("users")
                    .add_column(
                        ColumnDef::new("id_int")
                            .integer()
                            .not_null()
                            .auto_increment(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(Table::alter().table("users").drop_column("id").to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("users")
                    .rename_column("id_int", "id")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("users")
                    .modify_column(ColumnDef::new("id").primary_key())
                    .to_owned(),
            )
            .await
    }
}
