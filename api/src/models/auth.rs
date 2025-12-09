use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum Role {
    Initiator,
    Expert,
    ProjectOffice,
    Admin,
    Member,
    TeamLeader,
    TeamOwner,
    Teacher,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Initiator => write!(f, "INITIATOR"),
            Role::Expert => write!(f, "EXPERT"),
            Role::ProjectOffice => write!(f, "PROJECT_OFFICE"),
            Role::Admin => write!(f, "ADMIN"),
            Role::Member => write!(f, "MEMBER"),
            Role::TeamLeader => write!(f, "TEAM_LEADER"),
            Role::TeamOwner => write!(f, "TEAM_OWNER"),
            Role::Teacher => write!(f, "TEACHER"),
        }
    }
}

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
    pub roles: Vec<Role>,
    pub study_group: Option<String>,
    pub telephone: Option<String>,
}
