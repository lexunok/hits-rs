use sea_orm_migration::prelude::*;

use super::{m20251202_065032_create_user_table::Users, m20251225_134356_create_idea_table::Idea};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IdeaChecked::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(IdeaChecked::IdeaId).uuid().not_null())
                    .col(ColumnDef::new(IdeaChecked::UserId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(IdeaChecked::IdeaId)
                            .col(IdeaChecked::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IdeaChecked::Table, IdeaChecked::IdeaId)
                            .to(Idea::Table, Idea::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IdeaChecked::Table, IdeaChecked::UserId)
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
enum IdeaChecked {
    Table,
    IdeaId,
    UserId,
}
