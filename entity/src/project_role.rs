use sea_orm::entity::prelude::*;
use strum_macros::{Display, EnumString};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    EnumIter,
    EnumString,
    Display,
    DeriveActiveEnum,
    serde::Serialize,
    serde::Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectRole {
    #[sea_orm(string_value = "TEAM_LEADER")]
    TeamLeader,
    #[sea_orm(string_value = "INITIATOR")]
    Initiator,
    #[sea_orm(string_value = "MEMBER")]
    Member,
}
