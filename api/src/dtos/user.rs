use entity::role::Role;
use macros::IntoDataResponse;
use sea_orm::{DerivePartialModel, prelude::Uuid};
use serde::{Deserialize, Serialize};
use validator::Validate;

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
#[derive(IntoDataResponse, Debug, Serialize, Deserialize, Validate, DerivePartialModel)]
#[sea_orm(entity = "entity::users::Entity")]
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