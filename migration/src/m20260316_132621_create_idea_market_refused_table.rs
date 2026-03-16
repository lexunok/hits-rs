use sea_orm_migration::prelude::*;

use crate::{
    m20251225_134356_create_idea_table::Idea,
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
                    .table(IdeaMarketRefused::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(IdeaMarketRefused::TeamId).uuid().not_null())
                    .col(ColumnDef::new(IdeaMarketRefused::IdeaId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(IdeaMarketRefused::TeamId)
                            .col(IdeaMarketRefused::IdeaId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IdeaMarketRefused::Table, IdeaMarketRefused::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IdeaMarketRefused::Table, IdeaMarketRefused::IdeaId)
                            .to(Idea::Table, Idea::Id)
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
enum IdeaMarketRefused {
    Table,
    IdeaId,
    TeamId,
}
