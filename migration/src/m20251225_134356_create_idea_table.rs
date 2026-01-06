use sea_orm_migration::prelude::*;

use super::{
    m20251202_065032_create_user_table::Users, m20251221_103728_create_skill_table::Skill,
    m20251222_064711_create_groups_table::Group,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Idea::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Idea::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Idea::InitiatorId).uuid().not_null())
                    .col(ColumnDef::new(Idea::Name).string().not_null())
                    .col(ColumnDef::new(Idea::GroupExpertId).uuid().not_null())
                    .col(ColumnDef::new(Idea::GroupProjectOfficeId).uuid().not_null())
                    .col(ColumnDef::new(Idea::Status).string().not_null())
                    .col(
                        ColumnDef::new(Idea::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(Idea::Problem).string().not_null())
                    .col(ColumnDef::new(Idea::Solution).string().not_null())
                    .col(ColumnDef::new(Idea::Result).string().not_null())
                    .col(ColumnDef::new(Idea::Customer).string().not_null())
                    .col(ColumnDef::new(Idea::ContactPerson).string().not_null())
                    .col(ColumnDef::new(Idea::Description).string().not_null())
                    .col(ColumnDef::new(Idea::Suitability).big_unsigned().not_null())
                    .col(ColumnDef::new(Idea::Budget).big_unsigned().not_null())
                    .col(
                        ColumnDef::new(Idea::MaxTeamSize)
                            .small_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Idea::MinTeamSize)
                            .small_unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Idea::PreAssessment).double().not_null())
                    .col(
                        ColumnDef::new(Idea::Rating)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new(Idea::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Idea::ModifiedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Idea::Table, Idea::InitiatorId)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Idea::Table, Idea::GroupExpertId)
                            .to(Group::Table, Group::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Idea::Table, Idea::GroupProjectOfficeId)
                            .to(Group::Table, Group::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(IdeaSkill::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(IdeaSkill::IdeaId).uuid().not_null())
                    .col(ColumnDef::new(IdeaSkill::SkillId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(IdeaSkill::IdeaId)
                            .col(IdeaSkill::SkillId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IdeaSkill::Table, IdeaSkill::IdeaId)
                            .to(Idea::Table, Idea::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IdeaSkill::Table, IdeaSkill::SkillId)
                            .to(Skill::Table, Skill::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum Idea {
    Table,
    Id,
    InitiatorId,
    Name,
    GroupExpertId,
    GroupProjectOfficeId,
    Status,
    IsActive,
    Problem,
    Solution,
    Result,
    Customer,
    Description,
    ContactPerson,
    Suitability,
    Budget,
    MaxTeamSize,
    MinTeamSize,
    PreAssessment,
    Rating,
    CreatedAt,
    ModifiedAt,
}

#[derive(Iden)]
enum IdeaSkill {
    Table,
    IdeaId,
    SkillId,
}
