use crate::dtos::skill::SkillDto;
use entity::{idea_status::IdeaStatus, role::Role};
use macros::IntoDataResponse;
use sea_orm::prelude::{DateTimeLocal, DateTimeWithTimeZone, Uuid};
use serde::{Deserialize, Serialize};

#[derive(IntoDataResponse, Debug, Serialize, Deserialize)]
pub struct ProfileIdeaDto {
    pub id: Uuid,
    pub name: String,
    pub status: IdeaStatus,
}

#[derive(IntoDataResponse, Debug, Serialize, Deserialize, Default)]
pub struct TeamExperienceDto {
    pub team_id: Uuid,
    pub team_name: String,
    pub start_date: DateTimeWithTimeZone,
    pub finish_date: Option<DateTimeWithTimeZone>,
    pub has_active_project: bool,
}

#[derive(IntoDataResponse, Debug, Serialize, Deserialize, Default)]
pub struct ProfileDto {
    pub id: Uuid,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
    pub roles: Vec<Role>,
    pub email: String,
    pub last_name: String,
    pub first_name: String,
    pub created_at: DateTimeLocal,
    pub skills: Vec<SkillDto>,
    pub ideas: Vec<ProfileIdeaDto>,
    pub teams: Vec<TeamExperienceDto>,
}

#[derive(Debug, Deserialize)]
pub struct ProfileUpdatePayload {
    pub last_name: String,
    pub first_name: String,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
}
