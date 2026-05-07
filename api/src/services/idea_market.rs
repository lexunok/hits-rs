use std::collections::HashMap;

use crate::{
    AppState,
    dtos::{
        idea_market::{
            CreateIdeaMarketAdvertisementRequest, IdeaMarketAdvertisementDto,
            IdeaMarketAdvertisementQueryResult, IdeaMarketDto, IdeaMarketQueryResult,
            IdeaMarketTeamDto, IdeaMarketTeamQueryResult, IdeaSkillQueryResult,
            TeamSkillQueryResult,
        },
        skill::SkillDto,
        user::UserDto,
    },
    error::AppError,
};
use entity::{
    favorite_idea, idea, idea_market, idea_market_advertisement, idea_skill,
    idea_status::IdeaStatus, market,
    market_status::MarketStatus,
    prelude::{
        FavoriteIdea, IdeaMarket, IdeaMarketAdvertisement, IdeaSkill, Team, TeamMember,
        UserSkill,
    },
    request_status::RequestStatus,
    skill, team, team_member, user_skill, users,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, ExprTrait, IntoActiveModel, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set, TransactionTrait,
    prelude::Uuid,
    sea_query::{Alias, Expr, Query},
};

pub struct IdeaMarketService;

impl IdeaMarketService {
    async fn get_team_map(
        state: &AppState,
        team_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, IdeaMarketTeamDto>, AppError> {
        if team_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let owner = Alias::new("owner");
        let leader = Alias::new("leader");

        let teams: Vec<IdeaMarketTeamQueryResult> = Team::find()
            .filter(team::Column::Id.is_in(team_ids.iter().copied()))
            .join_as(
                JoinType::LeftJoin,
                team::Relation::TeamOwner.def(),
                owner.clone(),
            )
            .join_as(
                JoinType::LeftJoin,
                team::Relation::TeamLeader.def(),
                leader.clone(),
            )
            .tbl_col_as((owner.clone(), users::Column::Id), "owner_id")
            .tbl_col_as((owner.clone(), users::Column::Email), "owner_email")
            .tbl_col_as(
                (owner.clone(), users::Column::FirstName),
                "owner_first_name",
            )
            .tbl_col_as((owner.clone(), users::Column::LastName), "owner_last_name")
            .tbl_col_as((leader.clone(), users::Column::Id), "leader_id")
            .tbl_col_as((leader.clone(), users::Column::Email), "leader_email")
            .tbl_col_as(
                (leader.clone(), users::Column::FirstName),
                "leader_first_name",
            )
            .tbl_col_as((leader.clone(), users::Column::LastName), "leader_last_name")
            .column_as(
                Expr::expr(
                    Query::select()
                        .expr(Expr::val(1).count())
                        .from(TeamMember)
                        .and_where(
                            Expr::col(team_member::Column::TeamId)
                                .eq(Expr::col((Team, team::Column::Id))),
                        )
                        .and_where(Expr::col(team_member::Column::FinishDate).is_null())
                        .to_owned(),
                ),
                "members_count",
            )
            .into_model()
            .all(&state.conn)
            .await?;

        let team_skills: Vec<TeamSkillQueryResult> = UserSkill::find()
            .select_only()
            .column_as(team_member::Column::TeamId, "team_id")
            .column_as(skill::Column::Id, "id")
            .column_as(skill::Column::Name, "name")
            .column_as(skill::Column::SkillType, "type")
            .column_as(skill::Column::Confirmed, "confirmed")
            .column_as(skill::Column::CreatorId, "creator_id")
            .column_as(skill::Column::UpdaterId, "updater_id")
            .column_as(skill::Column::DeleterId, "deleter_id")
            .join(JoinType::InnerJoin, user_skill::Relation::Users.def())
            .join(JoinType::InnerJoin, users::Relation::TeamMember.def())
            .join(JoinType::InnerJoin, user_skill::Relation::Skill.def())
            .filter(team_member::Column::TeamId.is_in(team_ids.iter().copied()))
            .filter(team_member::Column::FinishDate.is_null())
            .into_model()
            .all(&state.conn)
            .await?;

        let mut skills_by_team: HashMap<Uuid, Vec<SkillDto>> = HashMap::new();
        for skill in team_skills {
            let entry = skills_by_team.entry(skill.team_id).or_default();
            if entry.iter().all(|item| item.id != skill.id) {
                entry.push(SkillDto {
                    id: skill.id,
                    name: skill.name,
                    skill_type: skill.skill_type,
                    confirmed: skill.confirmed,
                    creator_id: skill.creator_id,
                    updater_id: skill.updater_id,
                    deleter_id: skill.deleter_id,
                });
            }
        }

        Ok(teams
            .into_iter()
            .map(|team| {
                let owner = UserDto::from_parts(
                    team.owner_id,
                    team.owner_email.clone(),
                    team.owner_first_name.clone(),
                    team.owner_last_name.clone(),
                );

                let leader = UserDto::from_parts(
                    team.leader_id.unwrap_or(team.owner_id),
                    team.leader_email
                        .clone()
                        .unwrap_or_else(|| team.owner_email.clone()),
                    team.leader_first_name
                        .clone()
                        .unwrap_or_else(|| team.owner_first_name.clone()),
                    team.leader_last_name
                        .clone()
                        .unwrap_or_else(|| team.owner_last_name.clone()),
                );

                (
                    team.id,
                    IdeaMarketTeamDto {
                        id: team.id,
                        name: team.name,
                        owner,
                        leader,
                        members_count: team.members_count,
                        skills: skills_by_team.remove(&team.id).unwrap_or_default(),
                    },
                )
            })
            .collect())
    }

    async fn get_idea_skills_map(
        state: &AppState,
        idea_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<SkillDto>>, AppError> {
        if idea_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let skills: Vec<IdeaSkillQueryResult> = IdeaSkill::find()
            .select_only()
            .column(idea_skill::Column::IdeaId)
            .column_as(skill::Column::Id, "id")
            .column_as(skill::Column::Name, "name")
            .column_as(skill::Column::SkillType, "type")
            .column_as(skill::Column::Confirmed, "confirmed")
            .column_as(skill::Column::CreatorId, "creator_id")
            .column_as(skill::Column::UpdaterId, "updater_id")
            .column_as(skill::Column::DeleterId, "deleter_id")
            .join(JoinType::InnerJoin, idea_skill::Relation::Skill.def())
            .filter(idea_skill::Column::IdeaId.is_in(idea_ids.iter().copied()))
            .into_model()
            .all(&state.conn)
            .await?;

        let mut result: HashMap<Uuid, Vec<SkillDto>> = HashMap::new();
        for skill in skills {
            let entry = result.entry(skill.idea_id).or_default();
            if entry.iter().all(|item| item.id != skill.id) {
                entry.push(SkillDto {
                    id: skill.id,
                    name: skill.name,
                    skill_type: skill.skill_type,
                    confirmed: skill.confirmed,
                    creator_id: skill.creator_id,
                    updater_id: skill.updater_id,
                    deleter_id: skill.deleter_id,
                });
            }
        }

        Ok(result)
    }

    async fn get_all_by_filter(
        state: &AppState,
        user_id: Uuid,
        filter: Option<Condition>,
    ) -> Result<Vec<IdeaMarketDto>, AppError> {
        let rows: Vec<IdeaMarketQueryResult> = IdeaMarket::find()
            .join(JoinType::InnerJoin, idea_market::Relation::Idea.def())
            .join(JoinType::InnerJoin, idea::Relation::Users.def())
            .join(JoinType::InnerJoin, idea_market::Relation::Market.def())
            .column_as(users::Column::Id, "initiator_id")
            .column_as(users::Column::Email, "initiator_email")
            .column_as(users::Column::FirstName, "initiator_first_name")
            .column_as(users::Column::LastName, "initiator_last_name")
            .column_as(
                Expr::expr(
                    Query::select()
                        .expr(Expr::val(1).count())
                        .from(entity::prelude::TeamMarketRequest)
                        .and_where(
                            Expr::col(entity::team_market_request::Column::IdeaMarketId)
                                .equals((IdeaMarket, idea_market::Column::Id)),
                        )
                        .to_owned(),
                ),
                "requests",
            )
            .column_as(
                Expr::expr(
                    Query::select()
                        .expr(Expr::val(1).count())
                        .from(entity::prelude::TeamMarketRequest)
                        .and_where(
                            Expr::col(entity::team_market_request::Column::IdeaMarketId)
                                .equals((IdeaMarket, idea_market::Column::Id)),
                        )
                        .and_where(
                            Expr::col(entity::team_market_request::Column::Status)
                                .eq(RequestStatus::Accepted),
                        )
                        .to_owned(),
                ),
                "accepted_requests",
            )
            .column_as(
                Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(FavoriteIdea)
                        .and_where(Expr::col(favorite_idea::Column::UserId).eq(user_id))
                        .and_where(
                            Expr::col(favorite_idea::Column::IdeaMarketId)
                                .equals((IdeaMarket, idea_market::Column::Id)),
                        )
                        .to_owned(),
                ),
                "is_favorite",
            )
            .filter(
                Condition::all()
                    .add(market::Column::Status.eq(MarketStatus::Active))
                    .add_option(filter),
            )
            .order_by_desc(idea_market::Column::CreatedAt)
            .into_model()
            .all(&state.conn)
            .await?;

        let idea_ids: Vec<Uuid> = rows.iter().map(|row| row.idea_id).collect();
        let team_ids: Vec<Uuid> = rows.iter().filter_map(|row| row.team_id).collect();

        let skills_by_idea = Self::get_idea_skills_map(state, &idea_ids).await?;
        let teams_by_id = Self::get_team_map(state, &team_ids).await?;

        let mut items: Vec<IdeaMarketDto> = rows
            .into_iter()
            .map(|row| IdeaMarketDto {
                id: row.id,
                idea_id: row.idea_id,
                initiator: UserDto::from_parts(
                    row.initiator_id,
                    row.initiator_email,
                    row.initiator_first_name,
                    row.initiator_last_name,
                ),
                team: row.team_id.and_then(|team_id| teams_by_id.get(&team_id).cloned()),
                market_id: row.market_id,
                name: row.name,
                problem: row.problem,
                description: row.description,
                solution: row.solution,
                result: row.result,
                max_team_size: row.max_team_size,
                customer: row.customer,
                position: 0,
                stack: skills_by_idea.get(&row.idea_id).cloned().unwrap_or_default(),
                status: row.status,
                requests: row.requests,
                accepted_requests: row.accepted_requests,
                is_favorite: row.is_favorite,
            })
            .collect();

        items.sort_by(|left, right| {
            right
                .requests
                .cmp(&left.requests)
                .then_with(|| left.name.cmp(&right.name))
        });

        for (index, item) in items.iter_mut().enumerate() {
            item.position = index + 1;
        }

        Ok(items)
    }

    pub async fn get_all(state: &AppState, user_id: Uuid) -> Result<Vec<IdeaMarketDto>, AppError> {
        Self::get_all_by_filter(state, user_id, None).await
    }

    pub async fn get_all_by_market(
        state: &AppState,
        user_id: Uuid,
        market_id: Uuid,
    ) -> Result<Vec<IdeaMarketDto>, AppError> {
        Self::get_all_by_filter(state, user_id, Some(Condition::all().add(
            idea_market::Column::MarketId.eq(market_id),
        )))
        .await
    }

    pub async fn get_all_by_initiator(
        state: &AppState,
        user_id: Uuid,
        market_id: Uuid,
    ) -> Result<Vec<IdeaMarketDto>, AppError> {
        Self::get_all_by_filter(
            state,
            user_id,
            Some(
                Condition::all()
                    .add(idea_market::Column::MarketId.eq(market_id))
                    .add(idea::Column::InitiatorId.eq(user_id)),
            ),
        )
        .await
    }

    pub async fn get_all_favorite(
        state: &AppState,
        user_id: Uuid,
        market_id: Uuid,
    ) -> Result<Vec<IdeaMarketDto>, AppError> {
        Self::get_all_by_filter(
            state,
            user_id,
            Some(
                Condition::all()
                    .add(idea_market::Column::MarketId.eq(market_id))
                    .add(
                        Expr::exists(
                            Query::select()
                                .expr(Expr::val(1))
                                .from(FavoriteIdea)
                                .and_where(
                                    Expr::col(favorite_idea::Column::UserId).eq(user_id),
                                )
                                .and_where(
                                    Expr::col(favorite_idea::Column::IdeaMarketId)
                                        .equals((IdeaMarket, idea_market::Column::Id)),
                                )
                                .to_owned(),
                        ),
                    ),
            ),
        )
        .await
    }

    pub async fn get_one(
        state: &AppState,
        idea_market_id: Uuid,
        user_id: Uuid,
    ) -> Result<IdeaMarketDto, AppError> {
        Self::get_all_by_filter(
            state,
            user_id,
            Some(Condition::all().add(idea_market::Column::Id.eq(idea_market_id))),
        )
        .await?
        .into_iter()
        .next()
        .ok_or(AppError::NotFound)
    }

    pub async fn get_advertisements(
        state: &AppState,
        idea_market_id: Uuid,
    ) -> Result<Vec<IdeaMarketAdvertisementDto>, AppError> {
        let rows: Vec<IdeaMarketAdvertisementQueryResult> = IdeaMarketAdvertisement::find()
            .filter(idea_market_advertisement::Column::IdeaMarketId.eq(idea_market_id))
            .join(JoinType::InnerJoin, idea_market_advertisement::Relation::Users.def())
            .column_as(users::Column::Id, "sender_id")
            .column_as(users::Column::Email, "sender_email")
            .column_as(users::Column::FirstName, "sender_first_name")
            .column_as(users::Column::LastName, "sender_last_name")
            .order_by_asc(idea_market_advertisement::Column::CreatedAt)
            .into_model()
            .all(&state.conn)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| IdeaMarketAdvertisementDto {
                id: row.id,
                idea_market_id: row.idea_market_id,
                created_at: row.created_at,
                text: row.text,
                checked_by: row.checked_by,
                sender: UserDto::from_parts(
                    row.sender_id,
                    row.sender_email,
                    row.sender_first_name,
                    row.sender_last_name,
                ),
            })
            .collect())
    }

    pub async fn add_advertisement(
        state: &AppState,
        payload: CreateIdeaMarketAdvertisementRequest,
        sender: UserDto,
        is_admin: bool,
    ) -> Result<IdeaMarketAdvertisementDto, AppError> {
        let idea_market = IdeaMarket::find_by_id(payload.idea_market_id)
            .join(JoinType::InnerJoin, idea_market::Relation::Idea.def())
            .filter(
                Condition::all().add_option((!is_admin).then_some(
                    idea::Column::InitiatorId.eq(sender.id),
                )),
            )
            .one(&state.conn)
            .await?
            .ok_or(if is_admin {
                AppError::NotFound
            } else {
                AppError::Forbidden
            })?;

        let checked_by = vec![sender.email.clone()];

        let advertisement = idea_market_advertisement::ActiveModel {
            idea_market_id: Set(idea_market.id),
            sender_id: Set(sender.id),
            text: Set(payload.text),
            checked_by: Set(checked_by.clone()),
            ..Default::default()
        }
        .insert(&state.conn)
        .await?;

        Ok(IdeaMarketAdvertisementDto {
            id: advertisement.id,
            idea_market_id: advertisement.idea_market_id,
            created_at: advertisement.created_at.into(),
            text: advertisement.text,
            checked_by,
            sender,
        })
    }

    pub async fn mark_advertisement_checked(
        state: &AppState,
        advertisement_id: Uuid,
        email: String,
    ) -> Result<(), AppError> {
        let mut advertisement = IdeaMarketAdvertisement::find_by_id(advertisement_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        let mut checked_by = advertisement
            .checked_by
            .clone()
            .take()
            .unwrap_or_default();

        if checked_by.iter().all(|item| item != &email) {
            checked_by.push(email);
            advertisement.checked_by = Set(checked_by);
            advertisement.update(&state.conn).await?;
        }

        Ok(())
    }

    pub async fn add_to_favorite(
        state: &AppState,
        user_id: Uuid,
        idea_market_id: Uuid,
    ) -> Result<(), AppError> {
        let exists = FavoriteIdea::find_by_id((user_id, idea_market_id))
            .one(&state.conn)
            .await?;

        if exists.is_none() {
            favorite_idea::ActiveModel {
                user_id: Set(user_id),
                idea_market_id: Set(idea_market_id),
            }
            .insert(&state.conn)
            .await?;
        }

        Ok(())
    }

    pub async fn delete_from_favorite(
        state: &AppState,
        user_id: Uuid,
        idea_market_id: Uuid,
    ) -> Result<(), AppError> {
        FavoriteIdea::delete_by_id((user_id, idea_market_id))
            .exec(&state.conn)
            .await?;
        Ok(())
    }

    pub async fn delete(
        state: &AppState,
        idea_market_id: Uuid,
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<(), AppError> {
        let _ = IdeaMarket::find_by_id(idea_market_id)
            .join(JoinType::InnerJoin, idea_market::Relation::Idea.def())
            .filter(
                Condition::all().add_option((!is_admin).then_some(
                    idea::Column::InitiatorId.eq(user_id),
                )),
            )
            .one(&state.conn)
            .await?
            .ok_or(if is_admin {
                AppError::NotFound
            } else {
                AppError::Forbidden
            })?;

        IdeaMarket::delete_by_id(idea_market_id)
            .exec(&state.conn)
            .await?;

        Ok(())
    }

    pub async fn update_status(
        state: &AppState,
        idea_market_id: Uuid,
        status: entity::idea_market_status::IdeaMarketStatus,
    ) -> Result<(), AppError> {
        let mut idea_market = IdeaMarket::find_by_id(idea_market_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        idea_market.status = Set(status);
        idea_market.update(&state.conn).await?;

        Ok(())
    }

    pub async fn send_to_market(
        state: &AppState,
        market_id: Uuid,
        idea_ids: Vec<Uuid>,
    ) -> Result<Vec<IdeaMarketDto>, AppError> {
        let txn = state.conn.begin().await?;

        for idea_id in &idea_ids {
            let mut idea = entity::prelude::Idea::find_by_id(*idea_id)
                .one(&txn)
                .await?
                .ok_or(AppError::NotFound)?
                .into_active_model();

            idea.status = Set(IdeaStatus::OnMarket);
            idea.is_active = Set(true);
            idea.update(&txn).await?;

            idea_market::ActiveModel {
                idea_id: Set(*idea_id),
                market_id: Set(market_id),
                team_id: Set(None),
                status: Set(entity::idea_market_status::IdeaMarketStatus::RecruitmentIsOpen),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        txn.commit().await?;

        Self::get_all_by_market(state, Uuid::default(), market_id).await
    }

    pub async fn delete_advertisement(
        state: &AppState,
        advertisement_id: Uuid,
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<(), AppError> {
        let _ = IdeaMarketAdvertisement::find_by_id(advertisement_id)
            .filter(Condition::all().add_option((!is_admin).then_some(
                idea_market_advertisement::Column::SenderId.eq(user_id),
            )))
            .one(&state.conn)
            .await?
            .ok_or(if is_admin {
                AppError::NotFound
            } else {
                AppError::Forbidden
            })?;

        IdeaMarketAdvertisement::delete_by_id(advertisement_id)
            .exec(&state.conn)
            .await?;

        Ok(())
    }
}
