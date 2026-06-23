use crate::{
    AppState,
    dtos::{common::{MessageResponse, PaginatedResponse}, tag::{CreateTagRequest, TagDto, TagPaginationParams, UpdateTagRequest}},
    error::AppError,
    services::tag::TagService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn tag_router() -> Router<AppState> {
    Router::new()
        .route("/all", get(get_all_tags))
        .route("/add", post(create_confirmed_tag))
        .route("/add/no-confirmed", post(create_unconfirmed_tag))
        .route("/confirm/{tag_id}", put(confirm_tag))
        .route("/update/{tag_id}", put(update_tag))
        .route("/delete/{tag_id}", delete(delete_tag))
}

async fn get_all_tags(
    State(state): State<AppState>,
    _: Claims,
    Query(pagination): Query<TagPaginationParams>,
) -> Result<PaginatedResponse<TagDto>, AppError> {
    TagService::get_all(&state, pagination).await
}

#[has_any_role(Admin)]
async fn create_confirmed_tag(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateTagRequest>,
) -> Result<TagDto, AppError> {
    TagService::create_confirmed(&state, payload, claims.sub).await
}

#[has_any_role(Admin, ProjectOffice, Initiator, Member, TeamOwner, TeamLeader)]
async fn create_unconfirmed_tag(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateTagRequest>,
) -> Result<TagDto, AppError> {
    TagService::create_unconfirmed(&state, payload, claims.sub).await
}

#[has_any_role(Admin)]
async fn confirm_tag(
    State(state): State<AppState>,
    claims: Claims,
    Path(tag_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    TagService::confirm(&state, tag_id, claims.sub).await?;
    Ok(MessageResponse { message: "Тег утверждён".to_string() })
}

#[has_any_role(Admin)]
async fn update_tag(
    State(state): State<AppState>,
    claims: Claims,
    Path(tag_id): Path<Uuid>,
    Json(payload): Json<UpdateTagRequest>,
) -> Result<MessageResponse, AppError> {
    TagService::update(&state, tag_id, payload, claims.sub).await?;
    Ok(MessageResponse { message: "Тег обновлён успешно".to_string() })
}

#[has_any_role(Admin)]
async fn delete_tag(
    State(state): State<AppState>,
    claims: Claims,
    Path(tag_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    TagService::delete(&state, tag_id).await?;
    Ok(MessageResponse { message: "Тег удалён успешно".to_string() })
}
