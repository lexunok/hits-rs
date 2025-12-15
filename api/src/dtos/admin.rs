use entity::role::Role;
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
