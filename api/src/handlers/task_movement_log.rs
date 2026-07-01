use crate::{
    AppState,
    dtos::{common::{PaginatedResponse, PaginationParams}, task_movement_log::{MoveTaskRequest, TaskMovementLogDto}},
    error::AppError,
    services::task_movement_log::TaskMovementLogService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
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
    Query(params): Query<PaginationParams>
) -> Result<PaginatedResponse<TaskMovementLogDto>, AppError> {
    TaskMovementLogService::get_all_by_task(&state, task_id, params).await
}

async fn move_task(
    State(state): State<AppState>,
    _: Claims,
    Json(payload): Json<MoveTaskRequest>,
) -> Result<TaskMovementLogDto, AppError> {
    TaskMovementLogService::move_task(&state, payload).await
}
