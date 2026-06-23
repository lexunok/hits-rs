use crate::{
    AppState,
    dtos::{
        common::PaginatedResponse,
        market::{
            CreateMarketRequest, MarketDto, MarketPaginationParams, UpdateMarketRequest, UpdateMarketStatusRequest
        },
    },
    error::AppError,
};
use entity::{
    idea, idea_market, idea_market_status::IdeaMarketStatus, idea_status::IdeaStatus, market,
    market_status::MarketStatus, prelude::Market, team,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, Order, PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait, prelude::Uuid, sea_query::Expr
};

pub struct MarketService;

impl MarketService {
    pub async fn get_all(
        state: &AppState,
        pagination: MarketPaginationParams,
    ) -> Result<PaginatedResponse<MarketDto>, AppError> {

        let mut condition = Condition::all();
        let mut order = Order::Asc;

        if let Some(search_text) = pagination.search_text {
            condition = condition.add(market::Column::Name.ilike(format!("%{}%", search_text)));
        }
        if let Some(selected_statuses) = pagination.selected_statuses {
            condition = condition.add(market::Column::Status.is_in(selected_statuses));
        }
        if let Some(by_descending) = pagination.by_descending {
            if by_descending {
                order = Order::Desc
            }
        }

        let col = match pagination.order_by.as_deref() {
            Some("start_date") => market::Column::StartDate,
            Some("finish_date") => market::Column::FinishDate,
            _ => market::Column::Id
        };

        let query = Market::find()
            .filter(condition)
            .order_by(col, order)
            .into_partial_model();

        let paginator = query.paginate(&state.conn, pagination.page_size);
        let count = paginator.num_items().await.unwrap_or(0);
        let list = paginator.fetch_page(pagination.page).await.unwrap_or_default();

        Ok(PaginatedResponse { count, list })
    }

    pub async fn get_all_active(
        state: &AppState
    ) -> Vec<MarketDto> {
        Market::find()
            .filter(market::Column::Status.eq(MarketStatus::Active))
            .into_partial_model()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }

    pub async fn get_one(state: &AppState, id: Uuid) -> Result<MarketDto, AppError> {
        Market::find_by_id(id)
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn create(
        state: &AppState,
        payload: CreateMarketRequest,
    ) -> Result<MarketDto, AppError> {
        let market = payload.into_active_model().insert(&state.conn).await?;
        Ok(market_to_dto(market))
    }

    pub async fn update(
        state: &AppState,
        payload: UpdateMarketRequest,
    ) -> Result<MarketDto, AppError> {
        let market = payload.into_active_model().update(&state.conn).await?;
        Ok(market_to_dto(market))
    }

    pub async fn update_status(
        state: &AppState,
        payload: UpdateMarketStatusRequest,
        is_admin: bool,
    ) -> Result<MarketDto, AppError> {
        let market_id = payload.id;

        match payload.status {
            // Active — разрешено ProjectOffice и Admin
            MarketStatus::Active => {
                let market = payload.into_active_model().update(&state.conn).await?;
                Ok(market_to_dto(market))
            }
            // New — только Admin
            MarketStatus::New => {
                if !is_admin {
                    return Err(AppError::Forbidden);
                }
                let market = payload.into_active_model().update(&state.conn).await?;
                Ok(market_to_dto(market))
            }
            // Done — закрываем идеи без команды, удаляем открытые idea_market, сбрасываем has_active_project
            MarketStatus::Done => {
                let txn = state.conn.begin().await?;

                // 1. Идеи без команды в idea_market (RECRUITMENT_IS_OPEN) → CONFIRMED + is_active=false
                let open_idea_markets = entity::prelude::IdeaMarket::find()
                    .filter(idea_market::Column::MarketId.eq(market_id))
                    .filter(idea_market::Column::Status.eq(IdeaMarketStatus::RecruitmentIsOpen))
                    .all(&txn)
                    .await?;

                for im in &open_idea_markets {
                    entity::prelude::Idea::update_many()
                        .col_expr(idea::Column::Status, Expr::value(IdeaStatus::Confirmed))
                        .col_expr(idea::Column::IsActive, Expr::value(false))
                        .filter(idea::Column::Id.eq(im.idea_id))
                        .exec(&txn)
                        .await?;
                }

                // 2. Удаляем открытые idea_market записи
                entity::prelude::IdeaMarket::delete_many()
                    .filter(idea_market::Column::MarketId.eq(market_id))
                    .filter(idea_market::Column::Status.eq(IdeaMarketStatus::RecruitmentIsOpen))
                    .exec(&txn)
                    .await?;

                // 3. Команды с заявками на этот маркет → has_active_project=false
                entity::prelude::Team::update_many()
                    .col_expr(team::Column::HasActiveProject, Expr::value(false))
                    .filter(
                        team::Column::Id.in_subquery(
                            sea_orm::sea_query::Query::select()
                                .column(entity::team_market_request::Column::TeamId)
                                .from(entity::team_market_request::Entity)
                                .and_where(
                                    entity::team_market_request::Column::MarketId.eq(market_id),
                                )
                                .to_owned(),
                        ),
                    )
                    .exec(&txn)
                    .await?;

                // 4. Обновляем статус маркета
                let market = entity::prelude::Market::find_by_id(market_id)
                    .one(&txn)
                    .await?
                    .ok_or(AppError::NotFound)?
                    .into_active_model();
                let mut market_active = market;
                market_active.status = Set(MarketStatus::Done);
                let updated = market_active.update(&txn).await?;

                txn.commit().await?;

                Ok(market_to_dto(updated))
            }
        }
    }

    pub async fn delete(state: &AppState, id: Uuid) -> Result<(), AppError> {
        Market::delete_by_id(id).exec(&state.conn).await?;
        Ok(())
    }
}

fn market_to_dto(market: entity::market::Model) -> MarketDto {
    MarketDto {
        id: market.id,
        name: market.name,
        finish_date: market.finish_date,
        start_date: market.start_date,
        status: market.status,
    }
}
