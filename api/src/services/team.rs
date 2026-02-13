use crate::{
    AppState,
    dtos::{
        market::{CreateMarketRequest, MarketDto, UpdateMarketRequest, UpdateMarketStatusRequest},
        profile::UserDto,
        team::{CreateTeamRequest, TeamDto}, user,
    },
    error::AppError,
};
use entity::{
    market, market_status::MarketStatus, prelude::{Market, Team, TeamMember, TeamWantedSkill, Users}, role::Role, skill, team, team_member, team_wanted_skill, users
};
use sea_orm::{TransactionTrait,EntityLoaderTrait, Related,
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, ExprTrait, IntoActiveModel, JoinType, QueryFilter, QuerySelect, RelationTrait, prelude::Uuid, sea_query::{self, Expr}
};

pub struct TeamService;

impl TeamService {
    pub async fn get_all(state: &AppState, filter: Option<Expr>) -> Vec<MarketDto> {
        Market::find()
            .filter(Condition::all().add_option(filter))
            .into_partial_model()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }
    pub async fn get_all_active(state: &AppState) -> Vec<MarketDto> {
        Self::get_all(state, Some(market::Column::Status.eq(MarketStatus::Active))).await
    }
    pub async fn get_one(
        state: &AppState,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<TeamDto, AppError> {
        //         "tr.team_id as refused_team_id, " +
        //         "s.id as skill_id, s.name as skill_name, s.type as skill_type, " +
        //         "(SELECT EXISTS (SELECT 1 FROM team_member WHERE member_id = :userId AND finish_date IS NULL)) as existed_member " +
        //         "FROM team t " +
        //         "LEFT JOIN team_refused tr ON tr.user_id = :userId AND tr.team_id = t.id " +
        //         "LEFT JOIN user_skill us ON us.user_id = tm.member_id " +
        //         "LEFT JOIN skill s ON us.skill_id = s.id " +

        // let mut team: TeamDto = Team::find_by_id(team_id)
        //     .join_as(JoinType::LeftJoin, team::Relation::Leader.def(), "leader")
        //     .join_as(JoinType::LeftJoin, team::Relation::Owner.def(), "owner")
        //     .into_partial_model()
        //     .one(&state.conn)
        //     .await?
        //     .ok_or(AppError::NotFound)?;

        let team = Team::load()
            .filter_by_id(team_id)
            .with(team::Relation::Leader)
            .with(team::Relation::Owner)
            .with(team::Relation::Members)
            .with(skill::Entity)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        // let members: Vec<UserDto> = Users::find()
        //     .filter(
        //         users::Column::Id.in_subquery(
        //             sea_query::Query::select()
        //                 .column(team_member::Column::UserId)
        //                 .from(team_member::Entity)
        //                 .and_where(team_member::Column::TeamId.eq(team.id))
        //                 .to_owned(),
        //         ),
        //     )
        //     .into_partial_model()
        //     .all(&state.conn)
        //     .await?;

        Ok(team)
    }

    pub async fn create(
        state: &AppState,
        payload: CreateTeamRequest,
    ) -> Result<TeamDto, AppError> {
        let txn = state.conn.begin().await?;

        let team = team::ActiveModel {
            name: Set(payload.name),
            description: Set(payload.description),
            is_closed: Set(payload.is_closed),
            owner_id: Set(payload.owner_id),
            leader_id: Set(payload.leader_id),
            ..Default::default()
        }.insert(&txn).await?;

        let user_id = team.leader_id.unwrap_or_else(|| team.owner_id);

        let team_leader = Users::find_by_id(user_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut roles = team_leader.roles;

        if !roles.contains(&Role::TeamLeader) {
            roles.push(Role::TeamLeader);
            let team_leader = team_leader.into_active_model();
            team_leader.roles = Set(roles);
            team_leader.update(&txn).await?;
        };

        let members = payload
            .members
            .iter()
            .map(|m| team_member::ActiveModel {team_id: Set(team.id), user_id: Set(*m), ..Default::default()})
            .collect::<Vec<team_member::ActiveModel>>();

        let wanted_skills = payload
            .wanted_skills
            .iter()
            .map(|s| team_wanted_skill::ActiveModel {team_id: Set(team.id), skill_id: Set(*s)})
            .collect::<Vec<team_wanted_skill::ActiveModel>>();

        TeamMember::insert_many(members).exec(&txn).await?;
        TeamWantedSkill::insert_many(wanted_skills).exec(&txn).await?;

        txn.commit().await?;

        //ВЕРНУТЬ ТИМ ДТО
        Ok(TeamDto {

        })
    }
}
