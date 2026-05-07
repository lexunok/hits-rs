use crate::{
    AppState,
    dtos::{
        common::MessageResponse,
        project::TaskDto,
        task::{CreateTaskRequest, UpdateTaskRequest},
    },
    error::AppError,
    services::task::TaskService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post, put},
};
use sea_orm::prelude::Uuid;

pub fn task_router() -> Router<AppState> {
    Router::new()
        .route("/project/all/{project_id}", get(get_all_by_project))
        .route("/project/backlog/{project_id}", get(get_backlog))
        .route("/project/sprint/{sprint_id}", get(get_by_sprint))
        .route("/{task_id}", get(get_one))
        .route("/add", post(create_task))
        .route("/update/{task_id}", put(update_task))
        .route("/executor/{task_id}/{executor_id}", put(update_executor))
        .route("/move/{task_id}/{position}", put(move_task))
        .route("/leader/comment/{task_id}", put(update_leader_comment))
        .route("/executor/comment/{task_id}", put(update_executor_comment))
        .route("/delete/{task_id}", delete(delete_task))
}

async fn get_all_by_project(
    State(state): State<AppState>,
    _: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<TaskDto>>, AppError> {
    Ok(Json(TaskService::get_all_by_project(&state, project_id).await?))
}

async fn get_backlog(
    State(state): State<AppState>,
    _: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<TaskDto>>, AppError> {
    Ok(Json(TaskService::get_backlog(&state, project_id).await?))
}

async fn get_by_sprint(
    State(state): State<AppState>,
    _: Claims,
    Path(sprint_id): Path<Uuid>,
) -> Result<Json<Vec<TaskDto>>, AppError> {
    Ok(Json(TaskService::get_by_sprint(&state, sprint_id).await?))
}

async fn get_one(
    State(state): State<AppState>,
    _: Claims,
    Path(task_id): Path<Uuid>,
) -> Result<TaskDto, AppError> {
    TaskService::get_one(&state, task_id).await
}

async fn create_task(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<TaskDto, AppError> {
    TaskService::create(&state, payload, claims.sub).await
}

async fn update_task(
    State(state): State<AppState>,
    _: Claims,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<MessageResponse, AppError> {
    TaskService::update(&state, task_id, payload).await?;
    Ok(MessageResponse { message: "Задача успешно изменена".to_string() })
}

async fn update_executor(
    State(state): State<AppState>,
    _: Claims,
    Path((task_id, executor_id)): Path<(Uuid, Uuid)>,
) -> Result<MessageResponse, AppError> {
    TaskService::update_executor(&state, task_id, executor_id).await?;
    Ok(MessageResponse { message: "Новый исполнитель успешно назначен".to_string() })
}

async fn move_task(
    State(state): State<AppState>,
    _: Claims,
    Path((task_id, position)): Path<(Uuid, i32)>,
) -> Result<MessageResponse, AppError> {
    TaskService::move_position(&state, task_id, position).await?;
    Ok(MessageResponse { message: "Позиция обновлена".to_string() })
}

async fn update_leader_comment(
    State(state): State<AppState>,
    _: Claims,
    Path(task_id): Path<Uuid>,
    Json(comment): Json<String>,
) -> Result<MessageResponse, AppError> {
    TaskService::update_leader_comment(&state, task_id, comment).await?;
    Ok(MessageResponse { message: "Комментарий лидера обновлён".to_string() })
}

async fn update_executor_comment(
    State(state): State<AppState>,
    _: Claims,
    Path(task_id): Path<Uuid>,
    Json(comment): Json<String>,
) -> Result<MessageResponse, AppError> {
    TaskService::update_executor_comment(&state, task_id, comment).await?;
    Ok(MessageResponse { message: "Комментарий исполнителя обновлён".to_string() })
}

async fn delete_task(
    State(state): State<AppState>,
    _: Claims,
    Path(task_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    TaskService::delete(&state, task_id).await?;
    Ok(MessageResponse { message: "Задача успешно удалена".to_string() })
}
