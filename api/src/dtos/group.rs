use entity::role::Role;
use macros::IntoDataResponse;
use sea_orm::{DerivePartialModel, prelude::Uuid};
use serde::{Deserialize, Serialize};

use crate::dtos::user::UserDto;

#[derive(Serialize, IntoDataResponse, Debug, Deserialize, DerivePartialModel)]
#[sea_orm(entity = "entity::group::Entity")]
pub struct GroupDto {
    pub id: Uuid,
    pub name: String,
    pub roles: Vec<Role>,
    #[sea_orm(skip)]
    pub members: Vec<UserDto>,
}
#[derive(Deserialize, Debug)]
pub struct CreateGroupRequest {
    pub name: String,
    pub roles: Vec<Role>,
    pub members: Vec<Uuid>,
}
#[derive(Deserialize, Debug)]
pub struct UpdateGroupRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub roles: Option<Vec<Role>>,
    pub new_members: Option<Vec<Uuid>>,
    pub remove_members: Option<Vec<Uuid>>,
}

impl Default for GroupDto {
    fn default() -> Self {
        Self {
            id: Uuid::default(),
            name: String::default(),
            roles: vec![],
            members: vec![],
        }
    }
}
