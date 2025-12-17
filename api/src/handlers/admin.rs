use crate::{
    AppState,
    dtos::{
        admin::{InvitationPayload, UserCreatePayload, UserUpdatePayload},
        common::MessageResponse,
    },
    error::AppError,
    services::{invitation::InvitationService, user::UserService},
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, post, put},
};
use macros::has_role;
use sea_orm::prelude::Uuid;

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/invitations", post(send_invitations))
        .route("/users", post(create_user))
        .route("/users", put(update_user))
        .route("/users/restore/{email}", put(restore_user))
        .route("/users/{id}", delete(delete_user))
}

#[has_role(Admin)]
async fn send_invitations(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<InvitationPayload>,
) -> Result<MessageResponse, AppError> {
    let sent_count = InvitationService::send_invitations(&state, claims, payload).await?;

    if sent_count == 0 {
        return Ok(MessageResponse {
            message: "Все приглашения по указанным email уже были отправлены ранее.".to_string(),
        });
    }

    Ok(MessageResponse {
        message: format!(
            "Новые приглашения успешно отправлены в кол-ве {}",
            sent_count
        ),
    })
}

#[has_role(Admin)]
async fn create_user(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UserCreatePayload>,
) -> Result<MessageResponse, AppError> {
    UserService::create_user(&state, payload).await?;

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
    UserService::update_user(&state, payload).await?;

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
    UserService::restore_user(&state, email).await?;

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
    UserService::delete_user(&state, id).await?;

    Ok(MessageResponse {
        message: "Успешное удаление пользователя".to_string(),
    })
}
