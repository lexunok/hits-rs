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
pub enum TaskStatus {
    #[sea_orm(string_value = "InBackLog")]
    InBackLog,
    #[sea_orm(string_value = "OnModification")]
    OnModification,
    #[sea_orm(string_value = "NewTask")]
    NewTask,
    #[sea_orm(string_value = "InProgress")]
    InProgress,
    #[sea_orm(string_value = "OnVerification")]
    OnVerification,
    #[sea_orm(string_value = "Done")]
    Done,
}
