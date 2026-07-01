use entity::{role::Role, users};
use macros::IntoDataResponse;
use sea_orm::{
    DerivePartialModel,
    prelude::{DateTimeLocal, Uuid},
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::dtos::skill::SkillDto;

#[derive(Deserialize)]
pub struct UserPaginationParams {
    pub page: u64,
    pub page_size: u64,
    pub search_text: Option<String>,

    pub by_descending: Option<bool>,
    pub in_team: Option<bool>,
    pub ignored_team: Option<Uuid>,

    pub selected_roles: Option<Vec<Role>>,
    pub ignored_ids: Option<Vec<Uuid>>
}

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

impl UserDto {
    /// Создать минимальный UserDto из четырёх полей (используется в сервисах при JOIN-запросах).
    pub fn from_parts(id: Uuid, email: String, first_name: String, last_name: String) -> Self {
        Self {
            id,
            email,
            first_name,
            last_name,
            ..Default::default()
        }
    }

    /// Создать UserDto из Option-полей (для nullable JOIN-результатов).
    pub fn from_parts_opt(
        id: Option<Uuid>,
        email: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Option<Self> {
        id.map(|id| Self::from_parts(id, email.unwrap_or_default(), first_name.unwrap_or_default(), last_name.unwrap_or_default()))
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
