use crate::{
    AppState,
    dtos::{
        common::MessageResponse, group::GroupDto, skill::{CreateSkillRequest, SkillDto, UpdateSkillRequest}
    },
    error::AppError,
    services::{group::GroupService, skill::SkillService},
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get},
};
use entity::{role::Role, skill_type::SkillType};
use macros::has_role;
use sea_orm::prelude::Uuid;
use std::collections::HashMap;

pub fn group_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_skills).post(create_skill).put(update_skill))
        .route("/{id}", get(get_group_by_id).delete(delete_skill))
}

async fn get_all_skills(
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
) -> Result<(), AppError> {
    let group = GroupService::get_one(&state, id).await?;
    Ok(group)
}

#[has_role(Admin)]
async fn create_skill(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateSkillRequest>,
) -> Result<SkillDto, AppError> {
    let is_confirmed = claims.roles.contains(&Role::Admin);
    let skill = GroupService::create(&state, payload, claims.sub, is_confirmed).await?;
    Ok(skill)
}

#[has_role(Admin)]
async fn update_skill(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateSkillRequest>,
) -> Result<MessageResponse, AppError> {
    GroupService::update(&state, payload, claims.sub).await?;
    Ok(MessageResponse {
        message: "Группа успешно обновлена".to_string(),
    })
}

#[has_role(Admin)]
async fn delete_skill(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    GroupService::delete(&state, id, claims.sub).await?;
    Ok(MessageResponse {
        message: "Группа успешно удалена".to_string(),
    })
}
