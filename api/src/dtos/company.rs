use macros::IntoDataResponse;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use validator::Validate;
use super::profile::UserDto;

// Для запроса на создание
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCompanyRequest {
    #[validate(length(min = 1, message = "Название компании не может быть пустым"))]
    pub name: String,
    pub owner_id: Uuid,
    #[serde(default)]
    pub members: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyRequest {
    pub name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub members: Option<Vec<Uuid>>,
}

// Для ответа с детальной информацией (собирается в сервисе)
#[derive(IntoDataResponse, Debug, Serialize)]
pub struct CompanyDetailsResponse {
    pub id: Uuid,
    pub name: String,
    pub owner: UserDto,
    pub members: Vec<UserDto>,
}
