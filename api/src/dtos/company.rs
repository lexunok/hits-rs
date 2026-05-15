use super::user::UserDto;
use macros::IntoDataResponse;
use sea_orm::{DerivePartialModel, prelude::Uuid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateCompanyRequest {
    pub name: String,
    pub owner_id: Uuid,
    #[serde(default)]
    pub members: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub new_members: Option<Vec<Uuid>>,
    pub remove_members: Option<Vec<Uuid>>,
}

#[derive(IntoDataResponse, Debug, Serialize, Deserialize, DerivePartialModel)]
#[sea_orm(entity = "entity::company::Entity")]
pub struct CompanyResponse {
    pub id: Uuid,
    pub name: String,
    pub contact_person: String,
    #[sea_orm(nested)]
    pub owner: UserDto,
    #[sea_orm(skip)]
    pub members: Vec<UserDto>,
}

#[derive(IntoDataResponse, Debug, Serialize, Deserialize)]
pub struct CompanyDto {
    pub id: Uuid,
    pub name: String,
    pub contact_person: String
}