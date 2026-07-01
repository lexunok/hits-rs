pub use crate::dtos::tag::TagDto;
use crate::dtos::user::UserDto;
use entity::{
    project_role::ProjectRole, project_status::ProjectStatus, task_status::TaskStatus,
};
use macros::IntoDataResponse;
use sea_orm::{
    FromQueryResult,
    prelude::{Date, Uuid},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ProjectPaginationParams {
    pub page: u64,
    pub page_size: u64,
    pub search_text: Option<String>,
    pub selected_status: Option<ProjectStatus>
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct ProjectTeamDto {
    pub id: Uuid,
    pub name: String,
    pub members_count: i64,
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct TaskDto {
    pub id: Uuid,
    pub sprint_id: Option<Uuid>,
    pub project_id: Uuid,
    pub position: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub leader_comment: Option<String>,
    pub executor_comment: Option<String>,
    pub initiator: Option<UserDto>,
    pub executor: Option<UserDto>,
    pub work_hour: Option<f64>,
    pub start_date: Date,
    pub finish_date: Option<Date>,
    pub tags: Vec<TagDto>,
    pub status: Option<TaskStatus>,
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct ProjectMarksDto {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub project_role: ProjectRole,
    pub mark: Option<f64>,
    pub tasks: Vec<TaskDto>,
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct ReportProjectDto {
    pub project_id: Uuid,
    pub marks: Vec<ProjectMarksDto>,
    pub report: Option<String>,
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct ProjectMemberDto {
    pub user_id: Uuid,
    pub team_id: Option<Uuid>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub project_role: ProjectRole,
    pub start_date: Date,
    pub finish_date: Option<Date>,
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct ProjectDto {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub name: String,
    pub description: String,
    pub customer: String,
    pub initiator: UserDto,
    pub team: ProjectTeamDto,
    pub members: Vec<ProjectMemberDto>,
    pub report: ReportProjectDto,
    pub start_date: Date,
    pub finish_date: Option<Date>,
    pub status: ProjectStatus,
}

#[derive(Debug, Deserialize)]
pub struct AddToProjectRequest {
    pub team_id: Option<Uuid>,
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct FinishProjectRequest {
    pub report: String,
}

#[derive(Debug, FromQueryResult)]
pub struct ProjectBaseRow {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub report: Option<String>,
    pub start_date: Date,
    pub finish_date: Option<Date>,
    pub status: ProjectStatus,
    pub name: String,
    pub description: String,
    pub customer: String,
    pub initiator_id: Uuid,
    pub initiator_email: String,
    pub initiator_first_name: String,
    pub initiator_last_name: String,
    pub team_id: Uuid,
    pub team_name: String,
    pub team_members_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct ProjectMemberRow {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub team_id: Option<Uuid>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub project_role: ProjectRole,
    pub start_date: Date,
    pub finish_date: Option<Date>,
}

#[derive(Debug, FromQueryResult)]
pub struct ProjectMarksRow {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub mark: Option<f64>,
}

#[derive(Debug, FromQueryResult)]
pub struct ProjectTaskRow {
    pub id: Uuid,
    pub sprint_id: Option<Uuid>,
    pub project_id: Uuid,
    pub position: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub leader_comment: Option<String>,
    pub executor_comment: Option<String>,
    pub initiator_id: Option<Uuid>,
    pub initiator_email: Option<String>,
    pub initiator_first_name: Option<String>,
    pub initiator_last_name: Option<String>,
    pub executor_id: Option<Uuid>,
    pub executor_email: Option<String>,
    pub executor_first_name: Option<String>,
    pub executor_last_name: Option<String>,
    pub work_hour: Option<f64>,
    pub start_date: Date,
    pub finish_date: Option<Date>,
    pub status: Option<TaskStatus>,
}


