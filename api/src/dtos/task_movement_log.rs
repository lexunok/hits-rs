use crate::dtos::{project::TaskDto, user::UserDto};
use entity::task_status::TaskStatus;
use macros::IntoDataResponse;
use sea_orm::prelude::{DateTimeWithTimeZone, Uuid};
use serde::{Deserialize, Serialize};

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct TaskMovementLogDto {
    pub id: Uuid,
    pub task: TaskDto,
    pub executor: Option<UserDto>,
    pub user: Option<UserDto>,
    pub start_date: DateTimeWithTimeZone,
    pub end_date: Option<DateTimeWithTimeZone>,
    /// Строка вида "02ч 15мин" — вычисляется из разницы дат
    pub wasted_time: Option<String>,
    pub status: Option<TaskStatus>,
}

/// Запрос для смены статуса задачи (addNewTaskLog в старом коде).
#[derive(Debug, Deserialize)]
pub struct MoveTaskRequest {
    pub task_id: Uuid,
    pub executor_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub status: TaskStatus,
}
