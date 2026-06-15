use crate::{
    AppState,
    dtos::{
        common::{MessageResponse, PaginatedResponse},
        idea::IdeaDto,
        idea_market::{
            CreateIdeaMarketAdvertisementRequest, IdeaMarketAdvertisementDto, IdeaMarketDto, IdeaMarketPaginationParams,
        },
    },
    error::AppError,
    services::idea_market::IdeaMarketService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use entity::{idea_market_status::IdeaMarketStatus, role::Role};

use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn idea_market_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_idea_markets))
        .route("/send/{market_id}", put(send_ideas_to_market))
        .route("/favorite/{idea_market_id}", put(add_to_favorite).delete(delete_from_favorite))
        .route("/advertisement", post(add_advertisement))
        .route(
            "/advertisement/{idea_market_id}",
            get(get_advertisements_by_idea_market),
        )
        .route(
            "/advertisement/check/{advertisement_id}",
            put(mark_advertisement_checked),
        )
        .route(
            "/advertisement/{advertisement_id}/delete",
            delete(delete_advertisement),
        )
        .route("/status/{idea_market_id}/{status}", put(update_idea_market_status))
        .route("/{idea_market_id}", get(get_idea_market_by_id).delete(delete_idea_market))
}

async fn get_all_idea_markets(
    State(state): State<AppState>,
    claims: Claims,
    Query(pagination): Query<IdeaMarketPaginationParams>,
) -> Result<PaginatedResponse<IdeaMarketDto>, AppError> {
    IdeaMarketService::get_all(&state, claims.sub, pagination).await
}

async fn get_idea_market_by_id(
    State(state): State<AppState>,
    claims: Claims,
    Path(idea_market_id): Path<Uuid>,
) -> Result<IdeaMarketDto, AppError> {
    let item = IdeaMarketService::get_one(&state, idea_market_id, claims.sub).await?;
    Ok(item)
}

async fn get_advertisements_by_idea_market(
    State(state): State<AppState>,
    _claims: Claims,
    Path(idea_market_id): Path<Uuid>,
) -> Result<Json<Vec<IdeaMarketAdvertisementDto>>, AppError> {
    let items = IdeaMarketService::get_advertisements(&state, idea_market_id).await?;
    Ok(Json(items))
}

#[has_any_role(Admin, ProjectOffice)]
async fn send_ideas_to_market(
    State(state): State<AppState>,
    claims: Claims,
    Path(market_id): Path<Uuid>,
    Json(payload): Json<Vec<IdeaDto>>,
) -> Result<(), AppError> {
    IdeaMarketService::send_to_market(
        &state,
        market_id,
        payload.into_iter().map(|idea| idea.id).collect(),
    )
    .await
}

#[has_any_role(Admin, Initiator)]
async fn add_advertisement(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateIdeaMarketAdvertisementRequest>,
) -> Result<IdeaMarketAdvertisementDto, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    let item = IdeaMarketService::add_advertisement(
        &state,
        payload,
        crate::dtos::user::UserDto {
            id: claims.sub,
            email: claims.email,
            first_name: claims.first_name,
            last_name: claims.last_name,
            ..Default::default()
        },
        is_admin,
    )
    .await?;
    Ok(item)
}

async fn mark_advertisement_checked(
    State(state): State<AppState>,
    claims: Claims,
    Path(advertisement_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    IdeaMarketService::mark_advertisement_checked(&state, advertisement_id, claims.email).await?;
    Ok(MessageResponse {
        message: "Объявление обновлено".to_string(),
    })
}

async fn add_to_favorite(
    State(state): State<AppState>,
    claims: Claims,
    Path(idea_market_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    IdeaMarketService::add_to_favorite(&state, claims.sub, idea_market_id).await?;
    Ok(MessageResponse {
        message: "Идея добавлена в избранное".to_string(),
    })
}

async fn delete_from_favorite(
    State(state): State<AppState>,
    claims: Claims,
    Path(idea_market_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    IdeaMarketService::delete_from_favorite(&state, claims.sub, idea_market_id).await?;
    Ok(MessageResponse {
        message: "Идея удалена из избранного".to_string(),
    })
}

#[has_any_role(Admin, Initiator)]
async fn delete_idea_market(
    State(state): State<AppState>,
    claims: Claims,
    Path(idea_market_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    IdeaMarketService::delete(&state, idea_market_id, claims.sub, is_admin).await?;
    Ok(MessageResponse {
        message: "Идея удалена с маркета".to_string(),
    })
}

#[has_any_role(Admin, Initiator)]
async fn delete_advertisement(
    State(state): State<AppState>,
    claims: Claims,
    Path(advertisement_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    IdeaMarketService::delete_advertisement(&state, advertisement_id, claims.sub, is_admin)
        .await?;
    Ok(MessageResponse {
        message: "Объявление удалено".to_string(),
    })
}

#[has_any_role(Admin, Initiator, ProjectOffice)]
async fn update_idea_market_status(
    State(state): State<AppState>,
    claims: Claims,
    Path((idea_market_id, status)): Path<(Uuid, IdeaMarketStatus)>,
) -> Result<MessageResponse, AppError> {
    IdeaMarketService::update_status(&state, idea_market_id, status).await?;
    Ok(MessageResponse {
        message: "Статус идеи обновлен".to_string(),
    })
}
