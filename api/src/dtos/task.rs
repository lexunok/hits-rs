use sea_orm::prelude::{Date, Uuid};
use serde::Deserialize;

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
