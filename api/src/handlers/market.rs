use crate::{
    AppState,
    dtos::{
        common::{MessageResponse, PaginatedResponse},
        market::{CreateMarketRequest, MarketDto, MarketPaginationParams, UpdateMarketRequest, UpdateMarketStatusRequest},
    },
    error::AppError,
    services::market::MarketService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, put},
};
use entity::role::Role;
use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn market_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(get_all_markets).post(create_market).put(update_market),
        )
        .route("/active", get(get_active_markets))
        .route("/status", put(update_market_status))
        .route("/{id}", get(get_market_by_id).delete(delete_market))
}

async fn get_all_markets(
    State(state): State<AppState>,
    _: Claims,
    Query(pagination): Query<MarketPaginationParams>,
) -> Result<PaginatedResponse<MarketDto>, AppError> {
    MarketService::get_all(&state, pagination).await
}

async fn get_active_markets(State(state): State<AppState>, _: Claims) -> Json<Vec<MarketDto>> {
    let markets = MarketService::get_all_active(&state).await;
    Json(markets)
}

async fn get_market_by_id(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Result<MarketDto, AppError> {
    let market = MarketService::get_one(&state, id).await?;
    Ok(market)
}

#[has_any_role(Admin, ProjectOffice)]
async fn create_market(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateMarketRequest>,
) -> Result<MarketDto, AppError> {
    let market = MarketService::create(&state, payload).await?;
    Ok(market)
}

#[has_any_role(Admin, ProjectOffice)]
async fn update_market(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateMarketRequest>,
) -> Result<MarketDto, AppError> {
    let market = MarketService::update(&state, payload).await?;
    Ok(market)
}

#[has_any_role(Admin, ProjectOffice)]
async fn update_market_status(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateMarketStatusRequest>,
) -> Result<MarketDto, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);

    let market = MarketService::update_status(&state, payload, is_admin).await?;
    Ok(market)
}

#[has_any_role(Admin, ProjectOffice)]
async fn delete_market(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    MarketService::delete(&state, id).await?;
    Ok(MessageResponse {
        message: "Маркет успешно удален".to_string(),
    })
}
