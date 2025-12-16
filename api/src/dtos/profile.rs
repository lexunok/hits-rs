use macros::IntoDataResponse;
use sea_orm::{
    DerivePartialModel,
    prelude::{DateTimeWithTimeZone, Uuid},
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
    pub roles: Vec<String>,
    pub email: String,
    pub last_name: String,
    pub first_name: String,
    pub created_at: DateTimeWithTimeZone,
}
