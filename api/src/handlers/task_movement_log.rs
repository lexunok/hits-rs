use crate::{
    AppState,
    dtos::task_movement_log::{MoveTaskRequest, TaskMovementLogDto},
    error::AppError,
    services::task_movement_log::TaskMovementLogService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use sea_orm::prelude::Uuid;

pub fn task_movement_log_router() -> Router<AppState> {
    Router::new()
        .route("/all/{task_id}", get(get_all_by_task))
        .route("/add", post(move_task))
}

async fn get_all_by_task(
    State(state): State<AppState>,
    _: Claims,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Vec<TaskMovementLogDto>>, AppError> {
    let logs = TaskMovementLogService::get_all_by_task(&state, task_id).await?;
    Ok(Json(logs))
}

async fn move_task(
    State(state): State<AppState>,
    _: Claims,
    Json(payload): Json<MoveTaskRequest>,
) -> Result<TaskMovementLogDto, AppError> {
    TaskMovementLogService::move_task(&state, payload).await
}
