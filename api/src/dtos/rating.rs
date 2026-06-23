use crate::dtos::user::UserDto;
use entity::rating::ActiveModel;
use macros::IntoDataResponse;
use sea_orm::{DeriveIntoActiveModel, DerivePartialModel, prelude::Uuid};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize)]
pub struct RatingQuery {
    pub is_confirmed: bool
}


#[derive(Serialize, IntoDataResponse, Debug, DerivePartialModel)]
#[sea_orm(entity = "entity::rating::Entity")]
pub struct RatingDto {
    pub id: Uuid,
    #[sea_orm(nested)]
    pub expert: UserDto,
    pub idea_id: Uuid,
    pub market_value: i64,
    pub originality: i64,
    pub technical_realizability: i64,
    pub suitability: i64,
    pub budget: i64,
    pub rating: f64,
    pub is_confirmed: bool,
}

#[derive(Deserialize, Debug, DeriveIntoActiveModel, Validate)]
pub struct UpdateRatingRequest {
    pub id: Uuid,
    pub market_value: i64,
    pub originality: i64,
    pub technical_realizability: i64,
    pub suitability: i64,
    pub budget: i64,
}
