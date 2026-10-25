use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261025_050000_normalize_user_identifiers"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::update()
                    .table(Alias::new("users"))
                    .value(Alias::new("username"), Expr::cust("LOWER(username)"))
                    .value(Alias::new("email"), Expr::cust("LOWER(email)"))
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::update()
                    .table(Alias::new("backoffice_users"))
                    .value(Alias::new("username"), Expr::cust("LOWER(username)"))
                    .value(Alias::new("email"), Expr::cust("LOWER(email)"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
