use crate::{
    AppState,
    dtos::{
        common::MessageResponse,
        rating::{RatingDto, RatingQuery, UpdateRatingRequest},
    },
    error::AppError,
    services::rating::RatingService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, put},
};
use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn rating_router() -> Router<AppState> {
    Router::new()
        .route("/{idea_id}", get(get_all_ratings))
        .route("/save", put(save_rating))
}
async fn get_all_ratings(
    State(state): State<AppState>,
    _: Claims,
    Path(idea_id): Path<Uuid>,
) -> Json<Vec<RatingDto>> {
    let ratings = RatingService::get_all(&state, idea_id).await;
    Json(ratings)
}

#[has_any_role(Admin, Expert, ProjectOffice)]
async fn save_rating(
    State(state): State<AppState>,
    claims: Claims,
    Query(query): Query<RatingQuery>,
    Json(payload): Json<UpdateRatingRequest>,
) -> Result<MessageResponse, AppError> {
    if query.is_confirmed {
        RatingService::confirm(&state, payload).await?;
        Ok(MessageResponse {
            message: "Рейтинг успешно подтвержден".to_string(),
        })
    } else {
        RatingService::update(&state, payload).await?;
        Ok(MessageResponse {
            message: "Рейтинг успешно сохранен".to_string(),
        })
    }
}