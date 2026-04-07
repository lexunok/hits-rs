use crate::{
    AppState,
    dtos::{
        skill::SkillDto,
        team::{
            CreateTeamInvitation, CreateTeamRequest, TeamDto, TeamInvitationDto,
            TeamMarketRequestDto, UpdateTeamRequest,
        },
        user::UserDto,
    },
    error::AppError,
    utils::{security::Claims, smtp::send_team_invitation},
};
use chrono::{DateTime, Local, NaiveDateTime};
use entity::{
    idea, idea_market, idea_market_refused,
    prelude::{
        Idea, IdeaMarket, IdeaMarketRefused, Skill, Team, TeamInvitation, TeamMarketRequest,
        TeamMember, TeamRefused, TeamWantedSkill, UserSkill, Users,
    },
    role::Role,
    team,
    team_invitation::{self, ActiveModel},
    team_market_request, team_member, team_wanted_skill,
};

use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, Condition, EntityLoaderTrait, EntityTrait, ExprTrait, IntoActiveModel, JoinType,
    Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, SelectModel, Selector,
    TransactionTrait,
    prelude::Uuid,
    sea_query::{self, Expr},
};

pub struct TeamService;

impl TeamService {
    async fn build_basic_team_query(
        user_id: Uuid,
        filter: Option<Expr>,
        order: Option<(team::Column, Order)>,
        is_refused_expr: Option<Expr>,
    ) -> Selector<SelectModel<TeamDto>> {
        let owner = sea_query::Alias::new("owner");
        let leader = sea_query::Alias::new("leader");
        let is_refused_expr = is_refused_expr.unwrap_or(Expr::exists(
            sea_query::Query::select()
                .expr(Expr::val(1))
                .from(TeamMember)
                .and_where(
                    Expr::col(TeamMember::COLUMN.team_id).eq(Expr::col((Team, Team::COLUMN.id))),
                )
                .and_where(
                    Expr::col(TeamMember::COLUMN.user_id)
                        .eq(user_id)
                        .and(Expr::col(TeamMember::COLUMN.finish_date).is_null()),
                )
                .union(
                    sea_query::UnionType::All,
                    sea_query::Query::select()
                        .expr(Expr::val(1))
                        .from(TeamRefused)
                        .and_where(
                            Expr::col(TeamRefused::COLUMN.team_id)
                                .eq(Expr::col((Team, Team::COLUMN.id))),
                        )
                        .and_where(Expr::col(TeamRefused::COLUMN.team_id).eq(user_id))
                        .to_owned(),
                )
                .to_owned(),
        ));
        let mut query = Team::find()
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
            .join(JoinType::LeftJoin, team::Relation::TeamMember.def())
            .tbl_col_as((owner.clone(), Users::COLUMN.id), "owner_id")
            .tbl_col_as((owner.clone(), Users::COLUMN.email), "owner_email")
            .tbl_col_as(
                (owner.clone(), Users::COLUMN.first_name),
                "owner_first_name",
            )
            .tbl_col_as((owner.clone(), Users::COLUMN.last_name), "owner_last_name")
            .tbl_col_as((leader.clone(), Users::COLUMN.id), "leader_id")
            .tbl_col_as((leader.clone(), Users::COLUMN.email), "leader_email")
            .tbl_col_as(
                (leader.clone(), Users::COLUMN.first_name),
                "leader_first_name",
            )
            .tbl_col_as(
                (leader.clone(), Users::COLUMN.last_name),
                "leader_last_name",
            )
            .column_as(
                Expr::expr(
                    sea_query::Query::select()
                        .expr(Expr::val(1).count())
                        .from(TeamMember)
                        .and_where(
                            Expr::col(TeamMember::COLUMN.team_id)
                                .eq(Expr::col((Team, Team::COLUMN.id))),
                        )
                        .and_where(Expr::col(TeamMember::COLUMN.finish_date).is_null())
                        .to_owned(),
                ),
                "members_count",
            )
            .column_as(is_refused_expr, "is_refused")
            .filter(Condition::all().add_option(filter));
        if let Some((col, ord)) = order {
            query = query.order_by(col, ord);
        }
        query.into_model()
    }
    pub async fn get_all(state: &AppState, user_id: Uuid) -> Vec<TeamDto> {
        Self::build_basic_team_query(user_id, Some(Team::COLUMN.is_deleted.eq(false)), None, None)
            .await
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }
    pub async fn get_all_my(state: &AppState, user_id: Uuid, idea_id: Uuid) -> Vec<TeamDto> {
        // Здесь была логика что у TeamDto ставилось IsAcceptedToIdea если у него был активный проект или на паузе
        // Мне показалось что это тоже самое что поле team.has_active_project
        // В таком случае нужно будет при создании проекта не забыть обновить это поле у модели команды на true
        // Либо будет понятно почему было сделано так либо это был баг
        Self::build_basic_team_query(
            user_id,
            Some(
                Team::COLUMN.is_deleted.eq(false).and(
                    Team::COLUMN
                        .leader_id
                        .eq(user_id)
                        .or(Team::COLUMN.owner_id.eq(user_id)),
                ),
            ),
            Some((team::Column::HasActiveProject, Order::Desc)),
            Some(Expr::exists(
                sea_query::Query::select()
                    .expr(Expr::val(1))
                    .from(IdeaMarketRefused)
                    .and_where(
                        Expr::col(idea_market_refused::Column::TeamId)
                            .eq(Expr::col((Team, Team::COLUMN.id))),
                    )
                    .and_where(Expr::col(idea_market_refused::Column::IdeaId).eq(idea_id))
                    .to_owned(),
            )),
        )
        .await
        .all(&state.conn)
        .await
        .unwrap_or_default()
    }
    pub async fn get_one(
        state: &AppState,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<TeamDto, AppError> {
        let mut team: TeamDto =
            Self::build_basic_team_query(user_id, Some(Team::COLUMN.id.eq(team_id)), None, None)
                .await
                .one(&state.conn)
                .await?
                .ok_or(AppError::NotFound)?;

        let members: Vec<UserDto> = Users::find()
            .filter(
                Users::COLUMN.id.in_subquery(
                    sea_query::Query::select()
                        .column(TeamMember::COLUMN.user_id)
                        .from(TeamMember)
                        .and_where(TeamMember::COLUMN.team_id.eq(team_id))
                        .to_owned(),
                ),
            )
            .into_partial_model()
            .all(&state.conn)
            .await?;

        let wanted_skills: Vec<SkillDto> = Skill::find()
            .filter(
                Skill::COLUMN.id.in_subquery(
                    sea_query::Query::select()
                        .column(TeamWantedSkill::COLUMN.skill_id)
                        .from(TeamWantedSkill)
                        .and_where(TeamWantedSkill::COLUMN.team_id.eq(team_id))
                        .to_owned(),
                ),
            )
            .into_partial_model()
            .all(&state.conn)
            .await?;

        let member_skills: Vec<SkillDto> = Skill::find()
            .filter(
                Skill::COLUMN.id.in_subquery(
                    sea_query::Query::select()
                        .column(UserSkill::COLUMN.skill_id)
                        .from(UserSkill)
                        .and_where(
                            UserSkill::COLUMN.user_id.in_subquery(
                                sea_query::Query::select()
                                    .column(TeamMember::COLUMN.user_id)
                                    .from(TeamMember)
                                    .and_where(TeamMember::COLUMN.team_id.eq(team_id))
                                    .to_owned(),
                            ),
                        )
                        .to_owned(),
                ),
            )
            .into_partial_model()
            .all(&state.conn)
            .await?;

        team.wanted_skills = wanted_skills;
        team.members = members;
        team.member_skills = member_skills;

        Ok(team)
    }

    pub async fn get_team_invitations_by_user(
        state: &AppState,
        user_id: Uuid,
    ) -> Vec<TeamInvitationDto> {
        TeamInvitation::find()
            .filter(TeamInvitation::COLUMN.user_id.eq(user_id))
            .into_partial_model::<TeamInvitationDto>()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }
    pub async fn get_team_invitations_by_team(
        state: &AppState,
        team_id: Uuid,
    ) -> Vec<TeamInvitationDto> {
        TeamInvitation::find()
            .filter(TeamInvitation::COLUMN.team_id.eq(team_id))
            .into_partial_model::<TeamInvitationDto>()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }
    pub async fn get_team_market_requests(
        state: &AppState,
        team_id: Uuid,
    ) -> Vec<TeamMarketRequestDto> {
        TeamMarketRequest::find()
            .filter(TeamMarketRequest::COLUMN.team_id.eq(team_id))
            .left_join(IdeaMarket)
            .join(JoinType::InnerJoin, idea_market::Relation::Idea.def())
            .column_as(idea::Column::Name, "name")
            .into_model()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }
    pub async fn create(
        state: &AppState,
        payload: CreateTeamRequest,
        user_id: Uuid,
    ) -> Result<TeamDto, AppError> {
        let txn = state.conn.begin().await?;

        let members = payload
            .members
            .iter()
            .map(|m| team_member::ActiveModel::builder().set_user_id(*m))
            .collect::<Vec<team_member::ActiveModelEx>>();

        let wanted_skills = payload
            .wanted_skills
            .iter()
            .map(|s| team_wanted_skill::ActiveModel::builder().set_skill_id(*s))
            .collect::<Vec<team_wanted_skill::ActiveModelEx>>();

        let mut team = team::ActiveModel::builder()
            .set_name(payload.name)
            .set_description(payload.description)
            .set_is_closed(payload.is_closed)
            .set_owner_id(payload.owner_id)
            .set_leader_id(payload.leader_id);

        team.wanted_skills.replace_all(wanted_skills);
        team.team_members.replace_all(members);

        let team = team.insert(&txn).await?;

        let team_leader_id = team.leader_id.unwrap_or_else(|| team.owner_id);

        let team_leader = Users::find_by_id(team_leader_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut roles = team_leader.roles.clone();

        if !roles.contains(&Role::TeamLeader) {
            roles.push(Role::TeamLeader);
            let mut team_leader = team_leader.into_active_model();
            team_leader.roles = Set(roles);
            team_leader.update(&txn).await?;
        };

        txn.commit().await?;

        Self::get_one(state, team.id, user_id).await
    }
    pub async fn update(
        state: &AppState,
        payload: UpdateTeamRequest,
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<TeamDto, AppError> {
        let team_id = payload.id;

        if !is_admin {
            let _ = Team::find_by_id(team_id)
                .filter(
                    Team::COLUMN
                        .leader_id
                        .eq(user_id)
                        .or(Team::COLUMN.owner_id.eq(user_id)),
                )
                .one(&state.conn)
                .await?
                .ok_or(AppError::Forbidden)?;
        }

        let wanted_skills: Vec<team_wanted_skill::ActiveModelEx> = payload
            .wanted_skills
            .iter()
            .map(|id| {
                team_wanted_skill::ActiveModel {
                    team_id: Set(team_id),
                    skill_id: Set(*id),
                }
                .into_ex()
            })
            .collect();

        let mut team = payload.into_active_model().into_ex();

        // NOTE: update path kept as-is for now; team integration test currently reproduces a 500 here.
        team.wanted_skills.replace_all(wanted_skills);

        team.update(&state.conn).await?;

        Self::get_one(state, team_id, user_id).await
    }

    pub async fn send_invites_to_users(
        state: &AppState,
        invitations: Vec<CreateTeamInvitation>,
        claims: Claims,
    ) -> Result<(), AppError> {
        // МОЖНО ПРИГЛАСИТЬ ДАЖЕ НЕ В СВОЮ КОМАНДУ?
        let team = Team::find_by_id(invitations.get(0).map(|i| i.team_id).unwrap_or_default())
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let invitations: Vec<team_invitation::ActiveModel> = invitations
            .into_iter()
            .map(|i| i.into_active_model())
            .collect();

        let invitations = TeamInvitation::insert_many(invitations)
            .exec_with_returning(&state.conn)
            .await?;

        // Я предположил что отправляемый email это receiver
        for i in &invitations {
            send_team_invitation(
                team.id.to_string(),
                team.name.clone(),
                claims.first_name.clone(),
                claims.last_name.clone(),
                i.email.to_owned(),
            )
            .await
            .map_err(|_| {
                AppError::Custom(format!("Ошибка при отправке приглашения для {}", i.email))
            })?;
        }

        Ok(())
    }
    // Модели надо различать
    pub async fn create_team_request(
        state: &AppState,
        team_id: Uuid,
        claims: Claims,
    ) -> Result<(), AppError> {
        team_invitation::ActiveModel::builder()
            .set_email(claims.email)
            .set_first_name(claims.first_name)
            .set_last_name(claims.last_name)
            .set_team_id(team_id)
            .set_user_id(claims.sub)
            .insert(&state.conn)
            .await?;

        Ok(())
    }

    pub async fn add_team_member(
        state: &AppState,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<UserDto, AppError> {
        TeamMember::insert(team_member::ActiveModel {
            team_id: Set(team_id),
            user_id: Set(user_id),
            ..Default::default()
        })
        .exec(&state.conn)
        .await?;

        // ЕСЛИ ЕСТЬ ПРОЕКТ НУЖНО ЕЩЕ И В УЧАСТНИКИ ПРОЕКТА ДОБАВИТЬ

        let user: Vec<UserDto> = Users::load()
            .filter_by_id(user_id)
            .with(Skill)
            .all(&state.conn)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(user.into_iter().next().ok_or(AppError::NotFound)?)
    }

    pub async fn delete(
        state: &AppState,
        team_id: Uuid,
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<(), AppError> {
        let expr: Option<Expr> = if !is_admin {
            Some(Team::COLUMN.owner_id.eq(user_id))
        } else {
            None
        };

        let mut team = Team::find_by_id(team_id)
            .filter(Condition::all().add_option(expr))
            .one(&state.conn)
            .await?
            .ok_or(AppError::Forbidden)?
            .into_active_model();

        team.is_deleted = Set(true);

        let txn = state.conn.begin().await?;

        team.update(&txn).await?;

        TeamMember::update_many()
            .col_expr(
                team_member::Column::FinishDate,
                Expr::value(Some(Local::now())),
            )
            .col_expr(team_member::Column::IsActive, Expr::value(false))
            .filter(team_member::Column::TeamId.eq(team_id))
            .filter(team_member::Column::IsActive.eq(true))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(())
    }
}
