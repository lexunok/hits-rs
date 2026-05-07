use crate::dtos::project::TaskDto;
use entity::{project_role::ProjectRole, sprint_status::SprintStatus};
use macros::IntoDataResponse;
use sea_orm::prelude::{Date, Uuid};
use serde::{Deserialize, Serialize};

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct SprintDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub goal: String,
    pub report: Option<String>,
    pub start_date: Date,
    pub finish_date: Option<Date>,
    pub working_hours: Option<i64>,
    pub status: SprintStatus,
    pub tasks: Vec<TaskDto>,
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct SprintMarkDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub sprint_id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub project_role: ProjectRole,
    pub mark: Option<f64>,
    pub count_completed_tasks: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSprintRequest {
    pub project_id: Uuid,
    pub name: String,
    pub goal: String,
    pub working_hours: Option<i64>,
    pub start_date: Date,
    pub finish_date: Option<Date>,
    /// Задачи (id) которые переносятся в этот спринт из бэклога
    pub tasks: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSprintRequest {
    pub name: String,
    pub goal: Option<String>,
    pub working_hours: Option<i64>,
    pub start_date: Date,
    pub finish_date: Option<Date>,
    /// Итоговый список задач (id) этого спринта
    pub tasks: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AddSprintMarkRequest {
    pub user_id: Uuid,
    pub project_role: ProjectRole,
    pub mark: Option<f64>,
    /// Список id завершённых задач для подсчёта count_completed_tasks
    pub tasks: Vec<Uuid>,
}
