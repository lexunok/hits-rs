use sea_orm::entity::prelude::*;

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, serde::Serialize, serde::Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum SkillType {
    #[sea_orm(string_value = "Language")]
    Language,
    #[sea_orm(string_value = "Framework")]
    Framework,
    #[sea_orm(string_value = "Database")]
    Database,
    #[sea_orm(string_value = "Devops")]
    Devops,
}
