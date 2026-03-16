use crate::{
    AppState,
    dtos::market::{
        CreateMarketRequest, MarketDto, UpdateMarketRequest, UpdateMarketStatusRequest,
    },
    error::AppError,
};
use entity::{market, market_status::MarketStatus, prelude::Market};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter,
    prelude::Uuid, sea_query::Expr,
};

pub struct MarketService;

impl MarketService {
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
    pub async fn get_one(state: &AppState, id: Uuid) -> Result<MarketDto, AppError> {
        let market: MarketDto = Market::find_by_id(id)
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(market)
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
        } else if payload.status == MarketStatus::Done {
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
