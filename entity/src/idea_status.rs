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
pub enum IdeaStatus {
    #[sea_orm(string_value = "ON_EDITING")]
    OnEditing,
    #[sea_orm(string_value = "ON_APPROVAL")]
    OnApproval,
    #[sea_orm(string_value = "ON_CONFIRMATION")]
    OnConfirmation,
    #[sea_orm(string_value = "NEW")]
    New,
    #[sea_orm(string_value = "CONFIRMED")]
    Confirmed,
    #[sea_orm(string_value = "ON_MARKET")]
    OnMarket,
}
