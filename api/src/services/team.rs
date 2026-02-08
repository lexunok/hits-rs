use crate::{
    AppState,
    dtos::{
        market::{CreateMarketRequest, MarketDto, UpdateMarketRequest, UpdateMarketStatusRequest},
        profile::UserDto,
        team::TeamDto,
    },
    error::AppError,
};
use entity::{
    market,
    market_status::MarketStatus,
    prelude::{Market, Team, Users},
    team, team_member, users,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, JoinType, QueryFilter,
    QuerySelect, RelationTrait,
    prelude::Uuid,
    sea_query::{self, Expr},
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
        // String QUERY = "SELECT " +
        //         "tr.team_id as refused_team_id, " +
        //         "s.id as skill_id, s.name as skill_name, s.type as skill_type, " +
        //         "ws.id as wanted_skill_id, ws.name as wanted_skill_name, ws.type as wanted_skill_type," +
        //         "(SELECT COUNT(*) FROM team_member WHERE team_id = t.id AND finish_date IS NULL) as member_count, " +
        //         "(SELECT EXISTS (SELECT 1 FROM team_member WHERE member_id = :userId AND finish_date IS NULL)) as existed_member " +
        //         "FROM team t " +
        //         "LEFT JOIN team_refused tr ON tr.user_id = :userId AND tr.team_id = t.id " +
        //         "LEFT JOIN user_skill us ON us.user_id = tm.member_id " +
        //         "LEFT JOIN skill s ON us.skill_id = s.id " +
        //         "LEFT JOIN team_wanted_skill tws ON tws.team_id = t.id " +
        //         "LEFT JOIN skill ws ON tws.skill_id = ws.id " +
        //         "WHERE t.id = :teamId";

        let mut team: TeamDto = Team::find_by_id(team_id)
            .join_as(JoinType::LeftJoin, team::Relation::Leader.def(), "leader")
            .join_as(JoinType::LeftJoin, team::Relation::Owner.def(), "owner")
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let members: Vec<UserDto> = Users::find()
            .filter(
                users::Column::Id.in_subquery(
                    sea_query::Query::select()
                        .column(team_member::Column::UserId)
                        .from(team_member::Entity)
                        .and_where(team_member::Column::TeamId.eq(team.id))
                        .to_owned(),
                ),
            )
            .into_partial_model()
            .all(&state.conn)
            .await?;

        team.members = members;

        Ok(team)
    }

    pub async fn create(
        state: &AppState,
        payload: CreateMarketRequest,
    ) -> Result<MarketDto, AppError> {
        let market = payload.into_active_model().insert(&state.conn).await?;

        Ok(MarketDto {
            id: market.id,
            name: market.name,
            finish_date: market.finish_date,
            start_date: market.start_date,
            status: market.status,
        })
    }

    pub async fn update(
        state: &AppState,
        payload: UpdateMarketRequest,
    ) -> Result<MarketDto, AppError> {
        let market = payload.into_active_model().update(&state.conn).await?;

        Ok(MarketDto {
            id: market.id,
            name: market.name,
            finish_date: market.finish_date,
            start_date: market.start_date,
            status: market.status,
        })
    }

    pub async fn update_status(
        state: &AppState,
        payload: UpdateMarketStatusRequest,
        is_admin: bool,
    ) -> Result<MarketDto, AppError> {
        if payload.status == MarketStatus::Active || payload.status == MarketStatus::New && is_admin
        {
            let market = payload.into_active_model().update(&state.conn).await?;
            return Ok(MarketDto {
                id: market.id,
                name: market.name,
                finish_date: market.finish_date,
                start_date: market.start_date,
                status: market.status,
            });
        } else if payload.status == MarketStatus::Done && is_admin {
            //TODO: СДЕЛАТЬ ЧЕТО ОБНОВЛЕНИЕ ИДЕЙ, КОМАНД, ИДЕЯ МАРКЕТА И ТД
            //ЕЩЕ ПО РАСПИСАНИЮ ЗАПРОСЫ ЕСТЬ С ПОХОЖЕЙ ЛОГИКОЙ
            let market = payload.into_active_model().update(&state.conn).await?;
            return Ok(MarketDto {
                id: market.id,
                name: market.name,
                finish_date: market.finish_date,
                start_date: market.start_date,
                status: market.status,
            });
        }

        Err(AppError::Forbidden)
    }

    pub async fn delete(state: &AppState, id: Uuid) -> Result<(), AppError> {
        Market::delete_by_id(id).exec(&state.conn).await?;
        Ok(())
    }
}
