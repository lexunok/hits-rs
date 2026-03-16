use crate::{
    AppState,
    dtos::team::{CreateTeamRequest, TeamDto, UpdateTeamRequest},
    error::AppError,
    services::team::TeamService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use entity::role::Role;
use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn team_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_teams).post(create_team).put(update_team))
        .route("/my/{idea_id}", get(get_my_teams))
        .route("/{id}", get(get_team_by_id))
}

async fn get_teams(State(state): State<AppState>, claims: Claims) -> Json<Vec<TeamDto>> {
    Json(TeamService::get_all(&state, claims.sub).await)
}
async fn get_my_teams(
    State(state): State<AppState>,
    claims: Claims,
    Path(idea_id): Path<Uuid>,
) -> Json<Vec<TeamDto>> {
    Json(TeamService::get_all_my(&state, claims.sub, idea_id).await)
}
async fn get_team_by_id(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<TeamDto, AppError> {
    let team = TeamService::get_one(&state, id, claims.sub).await?;
    Ok(team)
}
#[has_any_role(Admin, TeamOwner)]
async fn create_team(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateTeamRequest>,
) -> Result<TeamDto, AppError> {
    let team = TeamService::create(&state, payload, claims.sub).await?;
    Ok(team)
}
#[has_any_role(Admin, TeamOwner, TeamLeader)]
async fn update_team(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateTeamRequest>,
) -> Result<TeamDto, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    let team = TeamService::update(&state, payload, claims.sub, is_admin).await?;
    Ok(team)
}
