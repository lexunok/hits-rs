use sea_orm_migration::prelude::*;

use crate::{m20251202_065032_create_user_table::Users, m20260208_190000_create_idea_market_related_tables::Team};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TeamInvitation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TeamInvitation::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(TeamInvitation::UserId).uuid().not_null())
                    .col(ColumnDef::new(TeamInvitation::TeamId).uuid().not_null())
                    .col(ColumnDef::new(TeamInvitation::Email).text().not_null())
                    .col(ColumnDef::new(TeamInvitation::FirstName).text().not_null())
                    .col(ColumnDef::new(TeamInvitation::LastName).text().not_null())
                    .col(
                        ColumnDef::new(TeamInvitation::Status)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TeamInvitation::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamInvitation::Table, TeamInvitation::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamInvitation::Table, TeamInvitation::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
}
#[derive(DeriveIden)]
enum TeamInvitation {
    Table,
    Id,
    UserId,
    TeamId,
    Email,
    FirstName,
    LastName,
    CreatedAt,
    Status
}