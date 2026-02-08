use crate::{
    AppState, dtos::team::TeamDto, error::AppError, services::team::TeamService,
    utils::security::Claims,
};
use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};
use sea_orm::prelude::Uuid;

pub fn team_router() -> Router<AppState> {
    Router::new().route("/{id}", get(get_team_by_id))
}

async fn get_team_by_id(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<TeamDto, AppError> {
    let team = TeamService::get_one(&state, id, claims.sub).await?;
    Ok(team)
}
