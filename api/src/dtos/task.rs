use entity::task_status::TaskStatus;
use sea_orm::prelude::{Date, Uuid};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct TaskPaginationParams {
    pub page: u64,
    pub page_size: u64,
    pub search_text: Option<String>,
    pub sprint_id: Option<Uuid>,
    pub project_id: Option<Uuid>,

    pub selected_statuses: Option<Vec<TaskStatus>>,
    pub selected_tags: Option<Vec<Uuid>>,
    pub selected_executors: Option<Vec<Uuid>>,
}
/// Запрос на создание задачи.
/// TaskDto для ответов живёт в dtos::project.
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub project_id: Uuid,
    pub sprint_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub work_hour: Option<f64>,
    pub start_date: Date,
    pub finish_date: Option<Date>,
    /// Теги (id)
    pub tags: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub work_hour: Option<f64>,
    pub tags: Vec<Uuid>,
}
