use crate::dtos::skill::SkillDto;
use crate::dtos::user::UserDto;
use entity::team::ActiveModel;
use macros::IntoDataResponse;
use sea_orm::{
    DeriveIntoActiveModel, FromQueryResult,
    prelude::{DateTimeLocal, Uuid},
};
use serde::{Deserialize, Serialize};

#[derive(IntoDataResponse, Serialize, Deserialize, FromQueryResult)]
pub struct OwnerDto {
    #[sea_orm(from_alias = "owner_id")]
    pub id: Uuid,
    #[sea_orm(from_alias = "owner_email")]
    pub email: String,
    #[sea_orm(from_alias = "owner_last_name")]
    pub last_name: String,
    #[sea_orm(from_alias = "owner_first_name")]
    pub first_name: String,
}
#[derive(IntoDataResponse, Serialize, Deserialize, FromQueryResult)]
pub struct LeaderDto {
    #[sea_orm(from_alias = "leader_id")]
    pub id: Uuid,
    #[sea_orm(from_alias = "leader_email")]
    pub email: String,
    #[sea_orm(from_alias = "leader_last_name")]
    pub last_name: String,
    #[sea_orm(from_alias = "leader_first_name")]
    pub first_name: String,
}
#[derive(IntoDataResponse, Serialize, Deserialize, FromQueryResult)]
pub struct TeamDto {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_closed: bool,
    pub has_active_project: bool,
    pub created_at: DateTimeLocal,

    #[sea_orm(nested, alias = "owner")]
    pub owner: OwnerDto,

    #[sea_orm(nested, alias = "leader")]
    pub leader: LeaderDto,

    #[sea_orm(skip)]
    pub members: Vec<UserDto>,

    #[sea_orm(skip)]
    pub wanted_skills: Vec<SkillDto>,

    #[sea_orm(skip)]
    pub member_skills: Vec<SkillDto>,

    pub members_count: i64,
    pub is_refused: bool,
}

#[derive(Deserialize, Debug)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: String,
    pub is_closed: bool,
    pub owner_id: Uuid,
    pub leader_id: Option<Uuid>,
    pub members: Vec<Uuid>,
    pub wanted_skills: Vec<Uuid>,
}

#[derive(IntoDataResponse, Debug, Serialize, Deserialize, DeriveIntoActiveModel)]
pub struct UpdateTeamRequest {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_closed: bool,
    #[sea_orm(skip)]
    pub wanted_skills: Vec<Uuid>,
}
