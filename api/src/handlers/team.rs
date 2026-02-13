use crate::{
    AppState, dtos::team::{CreateTeamRequest, TeamDto}, error::AppError, services::team::TeamService,
    utils::security::Claims,
};
use axum::{
    Json, Router, extract::{Path, State}, routing::{get, post}
};
use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn team_router() -> Router<AppState> {
    Router::new()
        .route("/",post(create_team))
        .route("/{id}", get(get_team_by_id))
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
    let team = TeamService::create(&state, payload).await?;
    Ok(team)
}
