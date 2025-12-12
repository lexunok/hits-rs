use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Invitation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Invitation::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")) 
                    )
                    .col(
                        ColumnDef::new(Invitation::Roles)
                            .array(ColumnType::String(StringLen::None))
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Invitation::Email)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Invitation::ExpiryDate)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP + INTERVAL '1 day'")),
                    )
                    .to_owned(),
            )
            .await
    }
}
#[derive(Iden)]
pub enum Invitation {
    Table,
    Id,
    ExpiryDate,
    Roles,
    Email,
}