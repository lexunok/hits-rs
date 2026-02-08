use super::{profile::UserDto, skill::SkillDto, team::TeamDto};
use entity::idea_market_status::IdeaMarketStatus;
use macros::IntoDataResponse;
use sea_orm::{
    DerivePartialModel, FromQueryResult,
    prelude::{DateTimeLocal, Uuid},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, IntoDataResponse, Debug, Deserialize, DerivePartialModel)]
#[sea_orm(entity = "entity::idea_market::Entity")]
pub struct IdeaMarketDto {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub market_id: Uuid,
    pub status: IdeaMarketStatus,
    #[sea_orm(skip)]
    pub team: Option<TeamDto>,
}
