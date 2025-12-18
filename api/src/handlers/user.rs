use crate::{
    AppState,
    dtos::{
        common::{MessageResponse, PaginationParams},
        profile::UserDto,
        user::{UserCreatePayload, UserUpdatePayload},
    },
    error::AppError,
    services::user::UserService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, put},
};
use macros::has_role;
use sea_orm::prelude::Uuid;

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_users).post(create_user).put(update_user))
        .route("/{id}", get(get_user).delete(delete_user))
        .route("/restore/{email}", put(restore_user))
}

async fn get_all_users(
    State(state): State<AppState>,
    _: Claims,
    Query(pagination): Query<PaginationParams>,
) -> Json<Vec<UserDto>> {
    Json(UserService::get_all(&state, pagination).await)
}
async fn get_user(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Result<UserDto, AppError> {
    UserService::get_one(&state, id).await
}

#[has_role(Admin)]
async fn create_user(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UserCreatePayload>,
) -> Result<MessageResponse, AppError> {
    UserService::create(&state, payload).await?;

    Ok(MessageResponse {
        message: "Успешное создание пользователя".to_string(),
    })
}

#[has_role(Admin)]
async fn update_user(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UserUpdatePayload>,
) -> Result<MessageResponse, AppError> {
    UserService::update(&state, payload).await?;

    Ok(MessageResponse {
        message: "Успешное обновление пользователя".to_string(),
    })
}

#[has_role(Admin)]
async fn restore_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(email): Path<String>,
) -> Result<MessageResponse, AppError> {
    UserService::restore(&state, email).await?;

    Ok(MessageResponse {
        message: "Успешное восстановление пользователя".to_string(),
    })
}

#[has_role(Admin)]
async fn delete_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    UserService::delete(&state, id).await?;

    Ok(MessageResponse {
        message: "Успешное удаление пользователя".to_string(),
    })
}
