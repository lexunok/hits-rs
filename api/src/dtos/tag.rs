use macros::IntoDataResponse;
use sea_orm::{DerivePartialModel, prelude::Uuid};
use serde::{Deserialize, Serialize};

#[derive(IntoDataResponse, Serialize, Deserialize, Debug, Clone, DerivePartialModel)]
#[sea_orm(entity = "entity::tag::Entity")]
pub struct TagDto {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub confirmed: bool,
    pub creator_id: Option<Uuid>,
    pub updater_id: Option<Uuid>,
    pub deleter_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTagRequest {
    pub name: String,
    pub color: String,
}
