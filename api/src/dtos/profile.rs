use crate::dtos::skill::SkillDto;
use entity::role::Role;
use macros::IntoDataResponse;
use sea_orm::prelude::{DateTimeLocal, Uuid};
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Deserialize)]
pub struct ProfileUpdatePayload {
    pub last_name: String,
    pub first_name: String,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
}
