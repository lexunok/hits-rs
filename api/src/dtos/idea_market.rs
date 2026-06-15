use crate::dtos::{skill::SkillDto, user::UserDto};
use entity::idea_market_status::IdeaMarketStatus;
use macros::IntoDataResponse;
use sea_orm::{
    FromQueryResult,
    prelude::{DateTimeLocal, Uuid},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct IdeaMarketPaginationParams {
    pub page: u64,
    pub page_size: u64,
    pub market_id: Option<Uuid>,
    pub favorite: Option<bool>,
    pub is_initiator: Option<bool>,
    pub search_text: Option<String>,
    pub selected_status: Option<IdeaMarketStatus>
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct IdeaMarketTeamDto {
    pub id: Uuid,
    pub name: String,
    pub owner: UserDto,
    pub leader: UserDto,
    pub members_count: i64,
    pub skills: Vec<SkillDto>,
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct IdeaMarketDto {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub initiator: UserDto,
    pub team: Option<IdeaMarketTeamDto>,
    pub market_id: Uuid,
    pub name: String,
    pub problem: String,
    pub description: String,
    pub solution: String,
    pub result: String,
    pub max_team_size: i16,
    pub customer: String,
    pub position: usize,
    pub stack: Vec<SkillDto>,
    pub status: IdeaMarketStatus,
    pub requests: i64,
    pub accepted_requests: i64,
    pub is_favorite: bool,
}

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone)]
pub struct IdeaMarketAdvertisementDto {
    pub id: Uuid,
    pub idea_market_id: Uuid,
    pub created_at: DateTimeLocal,
    pub text: String,
    pub sender: UserDto,
    pub checked_by: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIdeaMarketAdvertisementRequest {
    pub idea_market_id: Uuid,
    pub text: String,
}

#[derive(Debug, FromQueryResult)]
pub struct IdeaMarketQueryResult {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub market_id: Uuid,
    pub team_id: Option<Uuid>,
    pub status: IdeaMarketStatus,
    pub name: String,
    pub problem: String,
    pub description: String,
    pub solution: String,
    pub result: String,
    pub max_team_size: i16,
    pub customer: String,

    pub initiator_id: Uuid,
    pub initiator_email: String,
    pub initiator_first_name: String,
    pub initiator_last_name: String,

    pub requests: i64,
    pub accepted_requests: i64,
    pub is_favorite: bool,
}

#[derive(Debug, FromQueryResult)]
pub struct IdeaMarketTeamQueryResult {
    pub id: Uuid,
    pub name: String,

    pub owner_id: Uuid,
    pub owner_email: String,
    pub owner_first_name: String,
    pub owner_last_name: String,

    pub leader_id: Option<Uuid>,
    pub leader_email: Option<String>,
    pub leader_first_name: Option<String>,
    pub leader_last_name: Option<String>,

    pub members_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct IdeaSkillQueryResult {
    pub idea_id: Uuid,
    pub id: Uuid,
    pub name: String,
    #[sea_orm(from_alias = "type")]
    pub skill_type: entity::skill_type::SkillType,
    pub confirmed: bool,
    pub creator_id: Uuid,
    pub updater_id: Option<Uuid>,
    pub deleter_id: Option<Uuid>,
}

#[derive(Debug, FromQueryResult)]
pub struct TeamSkillQueryResult {
    pub team_id: Uuid,
    pub id: Uuid,
    pub name: String,
    #[sea_orm(from_alias = "type")]
    pub skill_type: entity::skill_type::SkillType,
    pub confirmed: bool,
    pub creator_id: Uuid,
    pub updater_id: Option<Uuid>,
    pub deleter_id: Option<Uuid>,
}

#[derive(Debug, FromQueryResult)]
pub struct IdeaMarketAdvertisementQueryResult {
    pub id: Uuid,
    pub idea_market_id: Uuid,
    pub created_at: DateTimeLocal,
    pub text: String,
    pub checked_by: Vec<String>,

    pub sender_id: Uuid,
    pub sender_email: String,
    pub sender_first_name: String,
    pub sender_last_name: String,
}
