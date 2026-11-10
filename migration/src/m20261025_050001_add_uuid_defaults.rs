use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261025_050001_add_uuid_defaults"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 三张表分属两个 schema，所以逐个列出来而不是用字符串循环
        for table in [
            ("identity", "customers"),
            ("identity", "operators"),
            ("merchant", "merchant_applications"),
        ] {
            manager
                .exec_stmt(
                    Table::alter()
                        .table(table)
                        .modify_column(
                            ColumnDef::new(Alias::new("id"))
                                .default(Expr::cust("gen_random_uuid()")),
                        )
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
