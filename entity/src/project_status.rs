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
pub enum ProjectStatus {
    #[sea_orm(string_value = "ACTIVE")]
    Active,
    #[sea_orm(string_value = "PAUSED")]
    Paused,
    #[sea_orm(string_value = "DONE")]
    Done,
    #[sea_orm(string_value = "DELETED")]
    Deleted,
    #[sea_orm(string_value = "FINISHED")]
    Finished,
}
