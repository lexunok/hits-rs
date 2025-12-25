use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sea_orm::prelude::Uuid;
use validator::Validate;

use crate::dtos::{group::GroupDto, profile::UserDto, skill::SkillDto};

use entity::idea::IdeaStatus;

#[derive(Serialize, Deserialize, Clone)]
pub struct IdeaResponse {
    pub id: Uuid,
    pub initiator: UserDto,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experts: Option<GroupDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_office: Option<GroupDto>,
    pub is_checked: bool,
    pub status: IdeaStatus,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_person: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suitability: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_assessment: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f64>,
    pub max_team_size: i16,
    pub min_team_size: i16,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CreateIdeaRequest {
    #[validate(length(min = 1))]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_expert_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_project_office_id: Option<Uuid>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_person: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub suitability: Option<i64>,
    pub budget: Option<i64>,
    pub max_team_size: i16,
    pub min_team_size: i16,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct UpdateIdeaRequest {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_expert_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_project_office_id: Option<Uuid>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_person: Option<String>,
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub suitability: Option<i64>,
    pub budget: Option<i64>,
    pub max_team_size: i16,
    pub min_team_size: i16,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct UpdateIdeaStatusRequest {
    pub status: IdeaStatus,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct IdeaSkillRequest {
    pub idea_id: Uuid,
    pub skills: Vec<SkillDto>,
}
