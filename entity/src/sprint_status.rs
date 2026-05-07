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
#[strum(serialize_all = "UPPERCASE")]
pub enum SprintStatus {
    #[sea_orm(string_value = "ACTIVE")]
    Active,
    #[sea_orm(string_value = "DONE")]
    Done,
}
