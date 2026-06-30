use crate::{
    AppState,
    dtos::{
        common::{MessageResponse, PaginatedResponse, PaginationParams},
        project::{
            FinishProjectRequest, ProjectDto, ProjectMarksDto, ProjectMemberDto,
            AddToProjectRequest,
        },
    },
    error::AppError,
    services::project::ProjectService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn project_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_projects))
        // .route("/my", get(get_my_projects))
        .route("/active", get(get_my_active_projects))
        .route("/create/{idea_market_id}", post(create_project))
        .route("/{project_id}", get(get_project_by_id).delete(delete_project))
        .route("/members/{project_id}", get(get_project_members).post(add_member))
        .route("/members/{project_id}/{user_id}", delete(kick_member_from_project_and_team))
        .route("/marks/{project_id}", get(get_project_marks))
        .route("/pause/{project_id}", put(pause_project))
        .route("/finish/{project_id}", put(finish_project))
        .route("/team/{project_id}/{team_id}", put(change_team_in_project))
}

async fn get_all_projects(
    State(state): State<AppState>,
    _claims: Claims,
    Query(pagination): Query<PaginationParams>,
) -> Result<PaginatedResponse<ProjectDto>, AppError> {
    ProjectService::get_all(&state, pagination).await
}

// async fn get_my_projects(
//     State(state): State<AppState>,
//     claims: Claims,
//     Query(pagination): Query<PaginationParams>,
// ) -> Result<PaginatedResponse<ProjectDto>, AppError> {
//     ProjectService::get_by_user(&state, claims.sub, pagination).await
// }

async fn get_my_active_projects(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<ProjectDto>>, AppError> {
    let projects = ProjectService::get_all_active(&state, claims.sub).await?;
    Ok(Json(projects))
}

async fn get_project_by_id(
    State(state): State<AppState>,
    _claims: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<ProjectDto, AppError> {
    let project = ProjectService::get_one(&state, project_id).await?;
    Ok(project)
}

async fn get_project_members(
    State(state): State<AppState>,
    _claims: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ProjectMemberDto>>, AppError> {
    let members = ProjectService::get_members(&state, project_id).await?;
    Ok(Json(members))
}

async fn get_project_marks(
    State(state): State<AppState>,
    _claims: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ProjectMarksDto>>, AppError> {
    let marks = ProjectService::get_marks(&state, project_id).await?;
    Ok(Json(marks))
}

#[has_any_role(Admin, ProjectOffice)]
async fn create_project(
    State(state): State<AppState>,
    claims: Claims,
    Path(idea_market_id): Path<Uuid>,
) -> Result<ProjectDto, AppError> {
    let project = ProjectService::create_from_idea_market(&state, idea_market_id).await?;
    Ok(project)
}

#[has_any_role(Admin, TeamLeader, TeamOwner, ProjectOffice)]
async fn add_member(
    State(state): State<AppState>,
    claims: Claims,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<AddToProjectRequest>,
) -> Result<ProjectMemberDto, AppError> {
    let member = ProjectService::add_member(&state, project_id, payload).await?;
    Ok(member)
}

#[has_any_role(Admin, TeamLeader, TeamOwner, ProjectOffice)]
async fn kick_member_from_project_and_team(
    State(state): State<AppState>,
    claims: Claims,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<MessageResponse, AppError> {
    ProjectService::kick_member_from_project_and_team(&state, project_id, user_id).await?;
    Ok(MessageResponse {
        message: "Участник удален из проекта".to_string(),
    })
}

#[has_any_role(Admin, TeamLeader, ProjectOffice)]
async fn pause_project(
    State(state): State<AppState>,
    claims: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    ProjectService::pause(&state, project_id).await?;
    Ok(MessageResponse {
        message: "Проект приостановлен".to_string(),
    })
}

#[has_any_role(Admin, TeamLeader, Initiator, ProjectOffice)]
async fn finish_project(
    State(state): State<AppState>,
    claims: Claims,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<FinishProjectRequest>,
) -> Result<MessageResponse, AppError> {
    ProjectService::finish(&state, project_id, payload.report, claims.sub, &claims.roles).await?;
    Ok(MessageResponse {
        message: "Проект завершен".to_string(),
    })
}

#[has_any_role(Admin, ProjectOffice)]
async fn change_team_in_project(
    State(state): State<AppState>,
    claims: Claims,
    Path((project_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<MessageResponse, AppError> {
    ProjectService::change_team(&state, project_id, team_id).await?;
    Ok(MessageResponse {
        message: "Команда проекта обновлена".to_string(),
    })
}

#[has_any_role(Admin, ProjectOffice)]
async fn delete_project(
    State(state): State<AppState>,
    claims: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    ProjectService::delete(&state, project_id).await?;
    Ok(MessageResponse {
        message: "Проект удален".to_string(),
    })
}
