use crate::{
    AppState,
    dtos::{
        common::MessageResponse,
        skill::{CreateSkillRequest, SkillDto, UpdateSkillRequest},
    },
    error::AppError,
    services::skill::SkillService,
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

pub fn skill_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_skills).post(create_skill).put(update_skill))
        .route("/{id}", delete(delete_skill))
        .route("/type/{skill_type}", get(get_skills_by_type))
        .route("/my",get(get_all_my_or_confirmed))
}

async fn get_all_skills(
    State(state): State<AppState>,
    _: Claims,
) -> Json<Vec<SkillDto>> {
    let skills = SkillService::get_all(&state).await;
    Json(skills)
}

async fn get_all_my_or_confirmed(
    State(state): State<AppState>,
    claims: Claims,
) -> Json<HashMap<String, Vec<SkillDto>>> {
    let skills = SkillService::get_all_my_or_confirmed(&state, claims.sub).await;
    Json(skills)
}

async fn get_skills_by_type(
    State(state): State<AppState>,
    _: Claims,
    Path(skill_type): Path<SkillType>,
) -> Json<Vec<SkillDto>> {
    let skills = SkillService::get_by_type(&state, skill_type).await;
    Json(skills)
}

async fn create_skill(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateSkillRequest>,
) -> Result<SkillDto, AppError> {
    let is_confirmed = claims.roles.contains(&Role::Admin);
    let skill = SkillService::create(&state, payload, claims.sub, is_confirmed).await?;
    Ok(skill)
}

#[has_role(Admin)]
async fn update_skill(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateSkillRequest>,
) -> Result<MessageResponse, AppError> {
    SkillService::update(&state, payload, claims.sub).await?;
    Ok(MessageResponse {
        message: "Навык успешно обновлен".to_string(),
    })
}

#[has_role(Admin)]
async fn delete_skill(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    SkillService::delete(&state, id, claims.sub).await?;
    Ok(MessageResponse {
        message: "Навык успешно удален".to_string(),
    })
}
