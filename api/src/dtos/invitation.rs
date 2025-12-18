use entity::role::Role;
use macros::IntoDataResponse;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct InvitationPayload {
    pub emails: Vec<String>,
    pub roles: Vec<Role>,
}

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct InvitationResponse {
    pub email: String,
    pub code: Uuid,
}