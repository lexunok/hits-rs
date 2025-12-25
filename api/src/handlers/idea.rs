use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use macros::has_role;
use sea_orm::prelude::Uuid;

use crate::{
    dtos::{
        common::MessageResponse,
        idea::{
            CreateIdeaRequest, IdeaResponse, IdeaSkillRequest, UpdateIdeaRequest,
            UpdateIdeaStatusRequest,
        },
        skill::SkillDto,
    },
    error::AppError,
    services::idea::IdeaService,
    utils::security::Claims,
    AppState,
};

pub fn idea_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_ideas).post(create_idea))
        .route("/draft", post(create_draft_idea))
        .route("/my", get(get_my_ideas))
        .route(
            "/:id",
            get(get_idea_by_id)
                .delete(delete_idea)
                .put(update_idea),
        )
        .route("/:id/send", put(send_idea_to_approval))
        .route("/:id/status", put(update_status))
        .route("/:id/skills", get(get_idea_skills))
        .route("/skills", put(update_idea_skills))
}

async fn get_idea_by_id(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<IdeaResponse>, AppError> {
    let idea = IdeaService::get_idea(&state, id, claims.sub).await?;
    Ok(Json(idea))
}

async fn get_all_ideas(
    State(state): State<AppState>,
    _: Claims,
) -> Result<Json<Vec<IdeaResponse>>, AppError> {
    let ideas = IdeaService::get_all(&state).await?;
    Ok(Json(ideas))
}

async fn get_my_ideas(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<IdeaResponse>>, AppError> {
    let ideas = IdeaService::get_list_by_initiator(&state, claims.sub).await?;
    Ok(Json(ideas))
}

async fn create_idea(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateIdeaRequest>,
) -> Result<Json<IdeaResponse>, AppError> {
    let idea = IdeaService::create(&state, payload, claims.sub).await?;
    // a little bit of duplication, but it's better than having a separate status in the request
    IdeaService::update_status_by_initiator(&state, idea.id, claims.sub).await?;
    let idea = IdeaService::get_idea(&state, idea.id, claims.sub).await?;
    Ok(Json(idea))
}

async fn create_draft_idea(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateIdeaRequest>,
) -> Result<Json<IdeaResponse>, AppError> {
    let idea = IdeaService::create(&state, payload, claims.sub).await?;
    Ok(Json(idea))
}

#[has_role(Admin)]
async fn update_idea(
    State(state): State<AppState>,
    _: Claims,
    Json(payload): Json<UpdateIdeaRequest>,
) -> Result<Json<IdeaResponse>, AppError> {
    let idea = IdeaService::update_by_admin(&state, payload).await?;
    Ok(Json(idea))
}

async fn delete_idea(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    if claims.roles.contains(&entity::role::Role::Admin) {
        IdeaService::delete_by_admin(&state, id).await?;
    } else {
        IdeaService::delete_by_initiator(&state, id, claims.sub).await?;
    }

    Ok(Json(MessageResponse {
        message: "Идея успешно удалена".to_string(),
    }))
}

async fn send_idea_to_approval(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    IdeaService::update_status_by_initiator(&state, id, claims.sub).await?;
    Ok(Json(MessageResponse {
        message: "Идея успешно отправлена на согласование".to_string(),
    }))
}

#[has_role(ProjectOffice, Expert, Admin)]
async fn update_status(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateIdeaStatusRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    IdeaService::update_status(&state, id, payload).await?;
    Ok(Json(MessageResponse {
        message: "Статус идеи успешно обновлен".to_string(),
    }))
}

async fn get_idea_skills(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SkillDto>>, AppError> {
    let skills = IdeaService::get_idea_skills(&state, id).await?;
    Ok(Json(skills))
}

async fn update_idea_skills(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<IdeaSkillRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let is_admin = claims.roles.contains(&entity::role::Role::Admin);
    IdeaService::update_idea_skills(&state, payload, claims.sub, is_admin).await?;
    Ok(Json(MessageResponse {
        message: "Навыки для идеи успешно обновлены".to_string(),
    }))
}
