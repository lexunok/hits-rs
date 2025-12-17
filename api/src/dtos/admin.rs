use entity::role::Role;
use macros::IntoDataResponse;
use sea_orm::{DerivePartialModel, prelude::{DateTimeLocal, Uuid}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct InvitationPayload {
    pub emails: Vec<String>,
    pub roles: Vec<Role>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
    pub last_name: String,
    pub first_name: String,
    pub roles: Vec<Role>,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
}
#[derive(IntoDataResponse, Debug, Serialize, Deserialize, DerivePartialModel)]
#[sea_orm(entity = "entity::users::Entity")]
pub struct UserUpdatePayload {
    pub id: Uuid,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
    pub roles: Vec<String>,
    pub password: String,        
    pub email: String,
    pub last_name: String,
    pub first_name: String,
    pub created_at: DateTimeLocal,
}
