use entity::{role::Role, users};
use macros::IntoDataResponse;
use sea_orm::{
    DerivePartialModel,
    prelude::{DateTimeLocal, Uuid},
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::dtos::skill::SkillDto;

impl From<users::ModelEx> for UserDto {
    fn from(u: users::ModelEx) -> Self {
        Self {
            id: u.id,
            first_name: u.first_name,
            last_name: u.last_name,
            email: u.email,
            skills: Some(u.skills.into_iter().map(Into::into).collect()),
            ..Default::default()
        }
    }
}

#[derive(IntoDataResponse, Debug, Serialize, Deserialize, Default, Clone, DerivePartialModel)]
#[sea_orm(entity = "entity::users::Entity")]
pub struct UserDto {
    pub id: Uuid,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
    pub roles: Vec<Role>,
    pub email: String,
    pub last_name: String,
    pub first_name: String,
    pub created_at: DateTimeLocal,
    #[sea_orm(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<SkillDto>>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct UserCreatePayload {
    pub email: String,
    pub password: String,
    pub last_name: String,
    pub first_name: String,
    pub roles: Vec<Role>,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
}
#[derive(IntoDataResponse, Debug, Serialize, Deserialize, Validate)]
pub struct UserUpdatePayload {
    pub id: Uuid,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
    pub roles: Vec<Role>,
    #[validate(email(message = "Некорректный формат email"))]
    pub email: String,
    pub last_name: String,
    pub first_name: String,
}
