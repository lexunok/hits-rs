use sea_orm_migration::prelude::*;

use crate::m20251202_065032_create_user_table::Users;
use crate::m20251221_103728_create_skill_table::Skill;
use crate::m20251225_134356_create_idea_table::Idea;
use crate::m20260115_123356_create_market_table::Market;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Team table
        manager
            .create_table(
                Table::create()
                    .table(Team::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Team::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Team::Name).string().not_null())
                    .col(ColumnDef::new(Team::Description).string().not_null())
                    .col(
                        ColumnDef::new(Team::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Team::OwnerId).uuid().not_null())
                    .col(ColumnDef::new(Team::LeaderId).uuid())
                    .col(ColumnDef::new(Team::MarketId).uuid())
                    .col(
                        ColumnDef::new(Team::HasActiveProject)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Team::IsDeleted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Team::IsClosed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Team::Table, Team::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Team::Table, Team::LeaderId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // TeamMember table
        manager
            .create_table(
                Table::create()
                    .table(TeamMember::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TeamMember::TeamId).uuid().not_null())
                    .col(ColumnDef::new(TeamMember::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(TeamMember::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(TeamMember::JoinDate)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(TeamMember::FinishDate).timestamp_with_time_zone())
                    .primary_key(
                        Index::create()
                            .col(TeamMember::TeamId)
                            .col(TeamMember::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamMember::Table, TeamMember::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamMember::Table, TeamMember::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // TeamWantedSkill table
        manager
            .create_table(
                Table::create()
                    .table(TeamWantedSkill::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TeamWantedSkill::TeamId).uuid().not_null())
                    .col(ColumnDef::new(TeamWantedSkill::SkillId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(TeamWantedSkill::TeamId)
                            .col(TeamWantedSkill::SkillId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamWantedSkill::Table, TeamWantedSkill::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamWantedSkill::Table, TeamWantedSkill::SkillId)
                            .to(Skill::Table, Skill::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // UserSkill table
        manager
            .create_table(
                Table::create()
                    .table(UserSkill::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserSkill::UserId).uuid().not_null())
                    .col(ColumnDef::new(UserSkill::SkillId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(UserSkill::UserId)
                            .col(UserSkill::SkillId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserSkill::Table, UserSkill::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserSkill::Table, UserSkill::SkillId)
                            .to(Skill::Table, Skill::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Project table
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Project::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(Project::IdeaId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Project::TeamId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Project::Status).string().not_null()) // ACTIVE, PAUSED, FINISHED
                    .col(
                        ColumnDef::new(Project::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Project::Table, Project::IdeaId)
                            .to(Idea::Table, Idea::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Project::Table, Project::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // IdeaMarket table
        manager
            .create_table(
                Table::create()
                    .table(IdeaMarket::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IdeaMarket::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(IdeaMarket::IdeaId).uuid().not_null())
                    .col(ColumnDef::new(IdeaMarket::MarketId).uuid().not_null())
                    .col(ColumnDef::new(IdeaMarket::TeamId).uuid())
                    .col(ColumnDef::new(IdeaMarket::Status).string().not_null()) // RECRUITMENT_IS_OPEN, RECRUITMENT_IS_CLOSED
                    .col(
                        ColumnDef::new(IdeaMarket::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IdeaMarket::Table, IdeaMarket::IdeaId)
                            .to(Idea::Table, Idea::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IdeaMarket::Table, IdeaMarket::MarketId)
                            .to(Market::Table, Market::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IdeaMarket::Table, IdeaMarket::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // FavoriteIdea table
        manager
            .create_table(
                Table::create()
                    .table(FavoriteIdea::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FavoriteIdea::UserId).uuid().not_null())
                    .col(ColumnDef::new(FavoriteIdea::IdeaMarketId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(FavoriteIdea::UserId)
                            .col(FavoriteIdea::IdeaMarketId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(FavoriteIdea::Table, FavoriteIdea::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(FavoriteIdea::Table, FavoriteIdea::IdeaMarketId)
                            .to(IdeaMarket::Table, IdeaMarket::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // TeamMarketRequest table
        manager
            .create_table(
                Table::create()
                    .table(TeamMarketRequest::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TeamMarketRequest::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(TeamMarketRequest::TeamId).uuid().not_null())
                    .col(
                        ColumnDef::new(TeamMarketRequest::MarketId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TeamMarketRequest::IdeaMarketId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TeamMarketRequest::Letter).text())
                    .col(
                        ColumnDef::new(TeamMarketRequest::Status)
                            .string()
                            .not_null(),
                    ) // NEW, ACCEPTED, CANCELED, WITHDRAWN, ANNULLED
                    .col(
                        ColumnDef::new(TeamMarketRequest::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamMarketRequest::Table, TeamMarketRequest::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamMarketRequest::Table, TeamMarketRequest::MarketId)
                            .to(Market::Table, Market::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TeamMarketRequest::Table, TeamMarketRequest::IdeaMarketId)
                            .to(IdeaMarket::Table, IdeaMarket::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // IdeaMarketAdvertisement table
        manager
            .create_table(
                Table::create()
                    .table(IdeaMarketAdvertisement::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IdeaMarketAdvertisement::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(IdeaMarketAdvertisement::IdeaMarketId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IdeaMarketAdvertisement::SenderId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IdeaMarketAdvertisement::Text)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IdeaMarketAdvertisement::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(IdeaMarketAdvertisement::CheckedBy)
                            .array(ColumnType::String(StringLen::None))
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                IdeaMarketAdvertisement::Table,
                                IdeaMarketAdvertisement::IdeaMarketId,
                            )
                            .to(IdeaMarket::Table, IdeaMarket::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                IdeaMarketAdvertisement::Table,
                                IdeaMarketAdvertisement::SenderId,
                            )
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
pub enum Team {
    Table,
    Id,
    Name,
    IsClosed,
    Description,
    CreatedAt,
    OwnerId,
    LeaderId,
    MarketId,
    HasActiveProject,
    IsDeleted,
}

#[derive(DeriveIden)]
enum TeamMember {
    Table,
    TeamId,
    UserId,
    IsActive,
    JoinDate,
    FinishDate,
}
#[derive(DeriveIden)]
enum TeamWantedSkill {
    Table,
    TeamId,
    SkillId,
}
#[derive(DeriveIden)]
enum UserSkill {
    Table,
    UserId,
    SkillId,
}

#[derive(DeriveIden)]
enum Project {
    Table,
    Id,
    IdeaId,
    TeamId,
    Status,
    CreatedAt,
}

#[derive(DeriveIden)]
enum IdeaMarket {
    Table,
    Id,
    IdeaId,
    MarketId,
    TeamId,
    Status,
    CreatedAt,
}

#[derive(DeriveIden)]
enum FavoriteIdea {
    Table,
    UserId,
    IdeaMarketId,
}

#[derive(DeriveIden)]
enum TeamMarketRequest {
    Table,
    Id,
    TeamId,
    MarketId,
    IdeaMarketId,
    Letter,
    Status,
    CreatedAt,
}

#[derive(DeriveIden)]
enum IdeaMarketAdvertisement {
    Table,
    Id,
    IdeaMarketId,
    SenderId,
    Text,
    CreatedAt,
    CheckedBy,
}
