use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000000_create_schemas"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 每条迁移都排在建表之前，所以这条必须是 migrations() 的第一个。
        // sea-query 没有 CREATE SCHEMA 的 builder，只能走裸 SQL。
        //
        // 三个 schema 对应三个领域：
        //   identity —— 顾客 + 运营（账号）
        //   merchant —— 商户 + 入驻审核
        //   catalog  —— 商品
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE SCHEMA IF NOT EXISTS identity;
                 CREATE SCHEMA IF NOT EXISTS merchant;
                 CREATE SCHEMA IF NOT EXISTS catalog;

                 -- 枚举类型跟着表进了各自 schema，而 sea-orm 拼 SQL 时会写
                 -- ::$1::application_status 这种不带 schema 的类型名 ——
                 -- 只有 search_path 能看到它才能解析。设在库级别，所有新连接都生效。
                 DO $$
                 BEGIN
                     EXECUTE format(
                         'ALTER DATABASE %I SET search_path TO identity, merchant, catalog, public',
                         current_database()
                     );
                 END $$;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP SCHEMA IF EXISTS catalog CASCADE;
                 DROP SCHEMA IF EXISTS merchant CASCADE;
                 DROP SCHEMA IF EXISTS identity CASCADE;",
            )
            .await?;
        Ok(())
    }
}
