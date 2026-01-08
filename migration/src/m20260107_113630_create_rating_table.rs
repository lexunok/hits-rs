use sea_orm_migration::prelude::*;

use crate::{m20251202_065032_create_user_table::Users, m20251225_134356_create_idea_table::Idea};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Rating::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Rating::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Rating::IdeaId).uuid().not_null())
                    .col(ColumnDef::new(Rating::ExpertId).uuid().not_null())
                    .col(
                        ColumnDef::new(Rating::Rating)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new(Rating::Budget)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Rating::MarketValue)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Rating::Originality)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Rating::TechnicalRealizability)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Rating::Suitability)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Rating::IsConfirmed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Rating::Table, Rating::IdeaId)
                            .to(Idea::Table, Idea::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Rating::Table, Rating::ExpertId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await
    }
}
#[derive(Iden)]
pub enum Rating {
    Table,
    Id,
    IdeaId,
    ExpertId,
    MarketValue,
    Originality,
    TechnicalRealizability,
    Suitability,
    Budget,
    Rating,
    IsConfirmed,
}
