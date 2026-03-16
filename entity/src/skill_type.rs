use sea_orm::entity::prelude::*;
use strum_macros::{Display, EnumString};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    EnumIter,
    Default,
    EnumString,
    Display,
    DeriveActiveEnum,
    serde::Serialize,
    serde::Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
#[strum(serialize_all = "UPPERCASE")]
pub enum SkillType {
    #[sea_orm(string_value = "LANGUAGE")]
    #[default]
    Language,
    #[sea_orm(string_value = "FRAMEWORK")]
    Framework,
    #[sea_orm(string_value = "DATABASE")]
    Database,
    #[sea_orm(string_value = "DEVOPS")]
    Devops,
}
