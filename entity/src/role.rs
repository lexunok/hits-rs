use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    EnumString,
    Display,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
#[strum(serialize_all = "UPPERCASE")]
pub enum Role {
    #[sea_orm(string_value = "INITIATOR")]
    Initiator,
    #[sea_orm(string_value = "EXPERT")]
    Expert,
    #[sea_orm(string_value = "PROJECT_OFFICE")]
    ProjectOffice,
    #[sea_orm(string_value = "ADMIN")]
    Admin,
    #[sea_orm(string_value = "MEMBER")]
    Member,
    #[sea_orm(string_value = "TEAM_LEADER")]
    TeamLeader,
    #[sea_orm(string_value = "TEAM_OWNER")]
    TeamOwner,
    #[sea_orm(string_value = "TEACHER")]
    Teacher,
}
