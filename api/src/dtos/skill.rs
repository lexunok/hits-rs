use entity::skill_type::SkillType;
use macros::IntoDataResponse;
use sea_orm::{prelude::Uuid, DerivePartialModel};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct CreateSkillRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub skill_type: SkillType,
}

#[derive(Deserialize, Debug)]
pub struct UpdateSkillRequest {
    pub id: Uuid,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub skill_type: Option<SkillType>,
}

#[derive(Serialize, IntoDataResponse, Debug, Clone, DerivePartialModel)]
#[sea_orm(entity = "entity::skill::Entity")]
pub struct SkillDto {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub skill_type: SkillType,
    pub confirmed: bool,
    pub creator_id: Uuid,
    pub updater_id: Option<Uuid>,
}
