use entity::{skill, skill_type::SkillType};
use macros::IntoDataResponse;
use sea_orm::{DerivePartialModel, prelude::Uuid};
use serde::{Deserialize, Serialize};

impl From<skill::ModelEx> for SkillDto {
    fn from(s: skill::ModelEx) -> Self {
        Self {
            id: s.id,
            name: s.name,
            skill_type: s.skill_type,
            confirmed: s.confirmed,
            ..Default::default()
        }
    }
}
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
    pub confirmed: Option<bool>,
}

#[derive(Serialize, Deserialize, IntoDataResponse, Debug, Clone, Default, DerivePartialModel)]
#[sea_orm(entity = "entity::skill::Entity")]
pub struct SkillDto {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub skill_type: SkillType,
    pub confirmed: bool,
    pub creator_id: Uuid,
    pub updater_id: Option<Uuid>,
    pub deleter_id: Option<Uuid>,
}
