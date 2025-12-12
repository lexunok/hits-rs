use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PasswordReset::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PasswordReset::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")) 
                    )
                    .col(
                        ColumnDef::new(PasswordReset::Code)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(PasswordReset::Email)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(PasswordReset::ExpiryDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PasswordReset::WrongTries)
                            .small_unsigned()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }
}
#[derive(Iden)]
pub enum PasswordReset {
    Table,
    Id,
    ExpiryDate,
    Code,
    WrongTries,
    Email,
}