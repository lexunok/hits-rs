use chrono::Local;
use entity::role::Role;
use macros::IntoDataResponse;
use sea_orm::{
    DerivePartialModel,
    prelude::{DateTimeLocal, Uuid},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ProfileUpdatePayload {
    pub last_name: String,
    pub first_name: String,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
}
#[derive(IntoDataResponse, Debug, Serialize, Deserialize, DerivePartialModel)]
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
}

impl Default for UserDto {
    fn default() -> Self {
        Self {
            id: Uuid::default(),
            study_group: None,
            telephone: None,
            roles: vec![],
            email: String::default(),
            last_name: String::default(),
            first_name: String::default(),
            created_at: Local::now(),
        }
    }
}
