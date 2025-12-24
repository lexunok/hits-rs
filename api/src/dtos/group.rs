use entity::role::Role;
use macros::IntoDataResponse;
use sea_orm::{prelude::Uuid, DerivePartialModel};
use serde::{Deserialize, Serialize};

use crate::dtos::profile::UserDto;

#[derive(Serialize, IntoDataResponse, Debug, DerivePartialModel)]
#[sea_orm(entity = "entity::group::Entity")]
pub struct GroupDto {
    pub id: Uuid,
    pub name: String,
    pub roles: Vec<Role>,
    #[sea_orm(skip)]
    pub members: Vec<UserDto>
}
#[derive(Deserialize, Debug)]
pub struct CreateGroupRequest {
    pub name: String,
    pub roles: Vec<Role>,
    pub members: Vec<Uuid>
}
#[derive(Deserialize, Debug)]
pub struct UpdateGroupRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub roles: Option<Vec<Role>>,
    pub members: Option<Vec<Uuid>>
}