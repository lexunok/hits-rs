use crate::{
    AppState,
    dtos::{
        skill::SkillDto,
        team::{CreateTeamRequest, TeamDto, UpdateTeamRequest},
        user::UserDto,
    },
    error::AppError,
};
use entity::{
    idea_market_refused,
    prelude::{
        IdeaMarketRefused, Skill, Team, TeamMember, TeamRefused, TeamWantedSkill, UserSkill, Users,
    },
    role::Role,
    team, team_member, team_wanted_skill,
};

use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    Condition, EntityTrait, ExprTrait, IntoActiveModel, JoinType, Order, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, SelectModel, Selector, TransactionTrait,
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

        team.wanted_skills.replace_all(wanted_skills);

        team.update(&state.conn).await?;

        Self::get_one(state, team_id, user_id).await
    }
}
