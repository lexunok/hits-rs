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
    routing::{delete, get, post, put},
};
use entity::skill_type::SkillType;
use macros::has_role;
use sea_orm::prelude::Uuid;
use std::collections::HashMap;

pub fn skill_router() -> Router<AppState> {
    Router::new()
        .route("/all", get(get_all_skills))
        .route(
            "/all-confirmed-or-creator",
            get(get_all_confirmed_or_creator),
        )
        .route("/by-type/:skill_type", get(get_skills_by_type))
        .route("/add", post(create_confirmed_skill))
        .route("/add/no-confirmed", post(create_unconfirmed_skill))
        .route("/update", put(update_skill))
        .route("/confirm/:id", put(confirm_skill))
        .route("/delete/:id", delete(delete_skill))
}

async fn get_all_skills(
    State(state): State<AppState>,
    _: Claims,
) -> Result<Json<Vec<SkillDto>>, AppError> {
    let skills = SkillService::get_all(&state).await?;
    Ok(Json(skills))
}

async fn get_all_confirmed_or_creator(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<HashMap<SkillType, Vec<SkillDto>>>, AppError> {
    let skills = SkillService::get_all_confirmed_or_creator(&state, claims.sub).await?;
    Ok(Json(skills))
}

async fn get_skills_by_type(
    State(state): State<AppState>,
    _: Claims,
    Path(skill_type): Path<SkillType>,
) -> Result<Json<Vec<SkillDto>>, AppError> {
    let skills = SkillService::get_by_type(&state, skill_type).await?;
    Ok(Json(skills))
}

async fn create_unconfirmed_skill(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateSkillRequest>,
) -> Result<Json<SkillDto>, AppError> {
    let skill = SkillService::create(&state, payload, claims.sub, false).await?;
    Ok(Json(skill))
}

#[has_role(Admin)]
async fn create_confirmed_skill(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateSkillRequest>,
) -> Result<Json<SkillDto>, AppError> {
    let skill = SkillService::create(&state, payload, claims.sub, true).await?;
    Ok(Json(skill))
}

#[has_role(Admin)]
async fn update_skill(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateSkillRequest>,
) -> Result<Json<SkillDto>, AppError> {
    let skill = SkillService::update(&state, payload, claims.sub).await?;
    Ok(Json(skill))
}

#[has_role(Admin)]
async fn confirm_skill(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<SkillDto>, AppError> {
    let skill = SkillService::confirm(&state, id, claims.sub).await?;
    Ok(Json(skill))
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
