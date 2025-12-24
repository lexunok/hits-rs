use crate::{
    AppState,
    dtos::{
        common::MessageResponse, group::{CreateGroupRequest, GroupDto, UpdateGroupRequest}
    },
    error::AppError,
    services::group::GroupService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use macros::has_role;
use sea_orm::prelude::Uuid;

pub fn group_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_groups).post(create_group).put(update_group))
        .route("/{id}", get(get_group_by_id).delete(delete_group))
}

async fn get_all_groups(
    State(state): State<AppState>,
    _: Claims,
) -> Json<Vec<GroupDto>> {
    let groups = GroupService::get_all(&state).await;
    Json(groups)
}

async fn get_group_by_id(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Result<GroupDto, AppError> {
    let group = GroupService::get_one(&state, id).await?;
    Ok(group)
}

#[has_role(Admin)]
async fn create_group(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<GroupDto, AppError> {
    let group = GroupService::create(&state, payload).await?;
    Ok(group)
}

#[has_role(Admin)]
async fn update_group(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateGroupRequest>,
) -> Result<GroupDto, AppError> {
    let group = GroupService::update(&state, payload).await?;
    Ok(group)
}

#[has_role(Admin)]
async fn delete_group(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    GroupService::delete(&state, id).await?;
    Ok(MessageResponse {
        message: "Группа успешно удалена".to_string(),
    })
}
