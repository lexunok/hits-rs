use crate::{
    AppState, dtos::{common::PaginationParams, profile::UserDto}, error::AppError, services::user::UserService, utils::security::Claims
};
use axum::{
    Json, Router,
    extract::{Path, Query, State}, routing::get,
};
use sea_orm::prelude::Uuid;

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_users))
        .route("/{id}", get(get_user))
}

async fn get_all_users(
    State(state): State<AppState>,
    _: Claims,
    Query(pagination): Query<PaginationParams>,
) -> Json<Vec<UserDto>> {
    Json(UserService::get_users(&state, pagination).await)
}
async fn get_user(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Result<UserDto, AppError> {
    UserService::get_user(&state, id).await
}


