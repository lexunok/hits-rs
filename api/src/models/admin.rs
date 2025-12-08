use serde::Deserialize;
use crate::models::auth::Role;

#[derive(Debug, Deserialize)]
pub struct InvitationPayload {
    pub emails: Vec<String>,
    pub roles: Vec<Role>,
}