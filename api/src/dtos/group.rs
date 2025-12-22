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
    pub users: Vec<UserDto>
}