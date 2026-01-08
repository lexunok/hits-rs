use crate::{
    AppState,
    dtos::{
        common::MessageResponse,
        rating::{RatingDto, UpdateRatingRequest},
    },
    error::AppError,
    services::rating::RatingService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, put},
};
use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn rating_router() -> Router<AppState> {
    Router::new()
        .route("/{idea_id}", get(get_all_ratings))
        .route("/expert/{idea_id}", get(get_all_ratings_by_expert))
        .route("/save", put(save_rating))
        .route("/confirm", put(confirm_rating))
}
async fn get_all_ratings(
    State(state): State<AppState>,
    _: Claims,
    Path(idea_id): Path<Uuid>,
) -> Json<Vec<RatingDto>> {
    let ratings = RatingService::get_all(&state, idea_id, None).await;
    Json(ratings)
}
async fn get_all_ratings_by_expert(
    State(state): State<AppState>,
    claims: Claims,
    Path(idea_id): Path<Uuid>,
) -> Json<Vec<RatingDto>> {
    let ratings = RatingService::get_all_by_expert(&state, claims.sub, idea_id).await;
    Json(ratings)
}

#[has_any_role(Admin, Expert, ProjectOffice)]
async fn save_rating(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateRatingRequest>,
) -> Result<MessageResponse, AppError> {
    RatingService::update(&state, payload).await?;
    Ok(MessageResponse {
        message: "Рейтинг успешно сохранен".to_string(),
    })
}

#[has_any_role(Admin, Expert, ProjectOffice)]
async fn confirm_rating(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateRatingRequest>,
) -> Result<MessageResponse, AppError> {
    RatingService::confirm(&state, payload).await?;
    Ok(MessageResponse {
        message: "Рейтинг успешно подтвержден".to_string(),
    })
}
