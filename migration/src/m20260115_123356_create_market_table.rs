use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Market::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Market::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Market::Name).string().not_null())
                    .col(ColumnDef::new(Market::StartDate).date().not_null())
                    .col(ColumnDef::new(Market::FinishDate).date().not_null())
                    .col(
                        ColumnDef::new(Market::Status)
                            .string()
                            .not_null()
                            .default("NEW"),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum Market {
    Table,
    Id,
    Name,
    StartDate,
    FinishDate,
    Status,
}
