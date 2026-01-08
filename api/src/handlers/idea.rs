use crate::{
    AppState,
    dtos::{
        common::{MessageResponse, PaginationParams},
        idea::{IdeaDto, IdeaSkillRequest, IdeaStatusRequest, IdeaWithChecked, SaveIdeaRequest},
        skill::SkillDto,
    },
    error::AppError,
    services::idea::IdeaService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post, put},
};
use entity::role::Role;
use macros::{has_any_role, has_role};
use sea_orm::prelude::Uuid;

pub fn idea_router() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(get_idea_by_id).delete(delete_idea))
        .route("/", get(get_all_ideas).post(save_idea))
        .route("/initiator", get(get_all_initiator_ideas))
        .route("/on-confirmation", get(get_all_on_confirmation_ideas))
        .route("/status", put(update_status))
        .route("/send/{id}", put(send_idea_to_approval))
        .route("/skills/{id}", get(get_idea_skills))
        .route("/skills", post(save_idea_skills))
}

async fn get_idea_by_id(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<IdeaWithChecked, AppError> {
    let idea = IdeaService::get_one(&state, id, claims.sub).await?;
    Ok(idea)
}
async fn get_all_ideas(
    State(state): State<AppState>,
    claims: Claims,
    Query(pagination): Query<PaginationParams>,
) -> Json<Vec<IdeaWithChecked>> {
    let ideas = IdeaService::get_all(&state, claims.sub, None, pagination).await;
    Json(ideas)
}
async fn get_all_initiator_ideas(
    State(state): State<AppState>,
    claims: Claims,
    Query(pagination): Query<PaginationParams>,
) -> Json<Vec<IdeaWithChecked>> {
    let ideas = IdeaService::get_all_by_initiator(&state, claims.sub, pagination).await;
    Json(ideas)
}
async fn get_all_on_confirmation_ideas(
    State(state): State<AppState>,
    claims: Claims,
    Query(pagination): Query<PaginationParams>,
) -> Json<Vec<IdeaWithChecked>> {
    let ideas = IdeaService::get_all_on_confirmation(&state, claims.sub, pagination).await;
    Json(ideas)
}
async fn get_idea_skills(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Json<Vec<SkillDto>> {
    let ideas = IdeaService::get_idea_skills(&state, id).await;
    Json(ideas)
}
#[has_any_role(Admin, Initiator)]
async fn save_idea(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<SaveIdeaRequest>,
) -> Result<IdeaDto, AppError> {
    let idea = if claims.roles.contains(&Role::Admin) {
        IdeaService::save(&state, payload, claims.sub, None).await?
    } else {
        IdeaService::save_by_initiator(&state, payload, claims.sub).await?
    };
    Ok(idea)
}
async fn save_idea_skills(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<IdeaSkillRequest>,
) -> Result<MessageResponse, AppError> {
    let initiator_id = Some(claims.sub).filter(|_| !claims.roles.contains(&Role::Admin));

    IdeaService::save_skills(&state, payload, initiator_id).await?;
    Ok(MessageResponse {
        message: "Навыки для идеи успешно обновлены".to_string(),
    })
}
#[has_role(Initiator)]
async fn send_idea_to_approval(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    IdeaService::update_status_by_initiator(&state, id, claims.sub).await?;
    Ok(MessageResponse {
        message: "Идея успешно отправлена на согласование".to_string(),
    })
}

#[has_any_role(ProjectOffice, Expert, Admin)]
async fn update_status(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<IdeaStatusRequest>,
) -> Result<MessageResponse, AppError> {
    IdeaService::update_status(&state, payload).await?;
    Ok(MessageResponse {
        message: "Статус идеи успешно обновлен".to_string(),
    })
}

#[has_any_role(Admin, Initiator)]
async fn delete_idea(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    if claims.roles.contains(&Role::Admin) {
        IdeaService::delete(&state, id, None).await?;
    } else {
        IdeaService::delete_by_initiator(&state, id, claims.sub).await?;
    }

    Ok(MessageResponse {
        message: "Идея успешно удалена".to_string(),
    })
}
