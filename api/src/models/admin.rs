use crate::models::auth::Role;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct InvitationPayload {
    pub emails: Vec<String>,
    pub roles: Vec<Role>,
}
