use crate::dtos::common::IntoApiResponse;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
    pub last_name: String,
    pub first_name: String,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub email: String,
    pub code: Uuid,
}
impl IntoApiResponse for InvitationResponse {}

#[derive(Debug, Deserialize)]
pub struct PasswordResetPayload {
    pub id: Uuid,
    pub code: String,
    pub password: String,
}
#[derive(Debug, Deserialize)]
pub struct EmailResetPayload {
    pub id: Uuid,
    pub code: String,
}
