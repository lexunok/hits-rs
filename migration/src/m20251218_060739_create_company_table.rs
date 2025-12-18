use sea_orm_migration::prelude::*;

use super::m20251202_065032_create_user_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Company::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Company::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Company::Name).string().not_null())
                    .col(ColumnDef::new(Company::OwnerId).uuid().not_null())
                    .col(
                        ColumnDef::new(Company::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Company::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Company::Table, Company::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CompanyMember::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CompanyMember::CompanyId).uuid().not_null())
                    .col(ColumnDef::new(CompanyMember::UserId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(CompanyMember::CompanyId)
                            .col(CompanyMember::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(CompanyMember::Table, CompanyMember::CompanyId)
                            .to(Company::Table, Company::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(CompanyMember::Table, CompanyMember::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

}

#[derive(Iden)]
enum Company {
    Table,
    Id,
    Name,
    OwnerId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum CompanyMember {
    Table,
    CompanyId,
    UserId,
}
