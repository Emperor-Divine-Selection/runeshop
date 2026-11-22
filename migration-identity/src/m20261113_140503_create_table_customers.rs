use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261113_140503_create_table_customers"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE customers (
                    id            uuid PRIMARY KEY,
                    created_at    timestamptz NOT NULL DEFAULT now(),
                    updated_at    timestamptz NOT NULL DEFAULT now(),
                    username      varchar(32)  NOT NULL UNIQUE,
                    password_hash varchar(255) NOT NULL,
                    email         varchar(255) NOT NULL UNIQUE
                )
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE customers")
            .await?;
        Ok(())
    }
}
