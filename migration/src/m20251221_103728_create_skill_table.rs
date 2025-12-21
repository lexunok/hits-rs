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
                    .table(Skill::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Skill::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Skill::Name).string().not_null())
                    .col(ColumnDef::new(Skill::SkillType).string().not_null())
                    .col(
                        ColumnDef::new(Skill::Confirmed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Skill::CreatorId).uuid().not_null())
                    .col(ColumnDef::new(Skill::UpdaterId).uuid())
                    .col(ColumnDef::new(Skill::DeleterId).uuid())
                    .col(
                        ColumnDef::new(Skill::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Skill::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Skill::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-skill-creator")
                            .from(Skill::Table, Skill::CreatorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-skill-updater")
                            .from(Skill::Table, Skill::UpdaterId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-skill-deleter")
                            .from(Skill::Table, Skill::DeleterId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum Skill {
    Table,
    Id,
    Name,
    #[iden = "skill_type"]
    SkillType,
    Confirmed,
    CreatorId,
    UpdaterId,
    DeleterId,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
