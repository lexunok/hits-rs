use crate::{
    AppState,
    dtos::{
        common::MessageResponse,
        sprint::{AddSprintMarkRequest, CreateSprintRequest, SprintDto, SprintMarkDto, UpdateSprintRequest},
    },
    error::AppError,
    services::sprint::SprintService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post, put},
};
use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn sprint_router() -> Router<AppState> {
    Router::new()
        .route("/project/{project_id}", get(get_all_sprints_by_project))
        .route("/project/{project_id}/active", get(get_active_sprint))
        .route("/{sprint_id}", get(get_sprint_by_id))
        .route("/add", post(create_sprint))
        .route("/{sprint_id}/update", put(update_sprint))
        .route("/{sprint_id}/finish", put(finish_sprint))
        .route("/{sprint_id}/delete", delete(delete_sprint))
        .route("/marks/{project_id}/{sprint_id}/add", post(add_sprint_marks))
        .route("/marks/{sprint_id}/all", get(get_sprint_marks))
}

async fn get_all_sprints_by_project(
    State(state): State<AppState>,
    _: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<SprintDto>>, AppError> {
    let sprints = SprintService::get_sprints_by_project(&state, project_id).await?;
    Ok(Json(sprints))
}

async fn get_sprint_by_id(
    State(state): State<AppState>,
    _: Claims,
    Path(sprint_id): Path<Uuid>,
) -> Result<Json<SprintDto>, AppError> {
    let sprint = SprintService::get_sprint_by_id(&state, sprint_id).await?;
    Ok(Json(sprint))
}

async fn get_active_sprint(
    State(state): State<AppState>,
    _: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Option<SprintDto>>, AppError> {
    let sprint = SprintService::get_active_sprint(&state, project_id).await?;
    Ok(Json(sprint))
}

async fn get_sprint_marks(
    State(state): State<AppState>,
    _: Claims,
    Path(sprint_id): Path<Uuid>,
) -> Result<Json<Vec<SprintMarkDto>>, AppError> {
    let marks = SprintService::get_sprint_marks(&state, sprint_id).await?;
    Ok(Json(marks))
}

#[has_any_role(Admin, ProjectOffice, TeamLeader)]
async fn create_sprint(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateSprintRequest>,
) -> Result<Json<SprintDto>, AppError> {
    let sprint = SprintService::create(&state, payload, claims.sub).await?;
    Ok(Json(sprint))
}

#[has_any_role(Admin, ProjectOffice, TeamLeader)]
async fn update_sprint(
    State(state): State<AppState>,
    claims: Claims,
    Path(sprint_id): Path<Uuid>,
    Json(payload): Json<UpdateSprintRequest>,
) -> Result<Json<SprintDto>, AppError> {
    let sprint = SprintService::update(&state, sprint_id, payload, claims.sub).await?;
    Ok(Json(sprint))
}

#[has_any_role(Admin, ProjectOffice, Initiator, TeamLeader)]
async fn add_sprint_marks(
    State(state): State<AppState>,
    claims: Claims,
    Path((project_id, sprint_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<Vec<AddSprintMarkRequest>>,
) -> Result<MessageResponse, AppError> {
    SprintService::add_marks(&state, sprint_id, project_id, payload).await?;
    Ok(MessageResponse {
        message: "Оценки успешно добавлены".to_string(),
    })
}

#[has_any_role(Admin, Initiator, TeamLeader)]
async fn finish_sprint(
    State(state): State<AppState>,
    claims: Claims,
    Path(sprint_id): Path<Uuid>,
    Json(report): Json<String>,
) -> Result<MessageResponse, AppError> {
    SprintService::finish(&state, sprint_id, report, claims.sub).await?;
    Ok(MessageResponse {
        message: "Спринт успешно завершён".to_string(),
    })
}

#[has_any_role(Admin, ProjectOffice, TeamLeader)]
async fn delete_sprint(
    State(state): State<AppState>,
    claims: Claims,
    Path(sprint_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    SprintService::delete(&state, sprint_id, claims.sub).await?;
    Ok(MessageResponse {
        message: "Спринт успешно удалён".to_string(),
    })
}
