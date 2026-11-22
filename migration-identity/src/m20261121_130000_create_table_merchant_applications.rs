use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261121_130000_create_table_merchant_applications"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 状态用 PG 原生 enum：类型本身就拒绝非法值，不靠 CHECK 兜底
        conn.execute_unprepared(
            "CREATE TYPE application_status AS ENUM ('pending', 'approved', 'rejected')",
        )
        .await?;

        conn.execute_unprepared(
            r#"
            CREATE TABLE merchant_applications (
                id          uuid PRIMARY KEY,
                created_at  timestamptz NOT NULL DEFAULT now(),
                updated_at  timestamptz NOT NULL DEFAULT now(),
                shop_name   varchar(64)  NOT NULL,
                username    varchar(32)  NOT NULL,
                email       varchar(255) NOT NULL,
                status      application_status NOT NULL DEFAULT 'pending',
                reviewed_by uuid,
                reviewed_at timestamptz,
                review_note text,

                CONSTRAINT chk_review_consistency CHECK (
                    (status = 'pending'  AND reviewed_by IS NULL     AND reviewed_at IS NULL)
                 OR (status <> 'pending' AND reviewed_by IS NOT NULL AND reviewed_at IS NOT NULL)
                )
            )
            "#,
        )
        .await?;

        // 同一个商户名 / 邮箱，同时只能有一条待审申请
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX uq_pending_username
             ON merchant_applications (username) WHERE status = 'pending'",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE UNIQUE INDEX uq_pending_email
             ON merchant_applications (email) WHERE status = 'pending'",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared("DROP TABLE merchant_applications")
            .await?;
        // 类型被列引用着，必须等表删完再删
        conn.execute_unprepared("DROP TYPE application_status")
            .await?;

        Ok(())
    }
}
