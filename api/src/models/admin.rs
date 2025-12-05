use serde::Deserialize;
use crate::models::auth::Role;

#[derive(Debug, Deserialize)]
pub struct InvitationPayload {
    pub email: Vec<String>,
    pub roles: Vec<Role>,
}