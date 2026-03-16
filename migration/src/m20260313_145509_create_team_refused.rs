use sea_orm_migration::prelude::*;

use crate::{
    m20251202_065032_create_user_table::Users,
    m20260208_190000_create_idea_market_related_tables::Team,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TeamRefused::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TeamRefused::TeamId).uuid().not_null())
                    .col(ColumnDef::new(TeamRefused::UserId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(TeamRefused::TeamId)
                            .col(TeamRefused::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamRefused::Table, TeamRefused::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamRefused::Table, TeamRefused::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
#[derive(DeriveIden)]
enum TeamRefused {
    Table,
    TeamId,
    UserId,
}
