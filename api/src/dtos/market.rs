use chrono::NaiveDate;
use entity::{market::ActiveModel, market_status::MarketStatus};
use macros::IntoDataResponse;
use sea_orm::{DeriveIntoActiveModel, DerivePartialModel, prelude::Uuid};
use serde::{Deserialize, Serialize};

#[derive(Serialize, IntoDataResponse, Debug, Deserialize, DerivePartialModel)]
#[sea_orm(entity = "entity::market::Entity")]
pub struct MarketDto {
    pub id: Uuid,
    pub name: String,
    pub start_date: NaiveDate,
    pub finish_date: NaiveDate,
    pub status: MarketStatus,
}
#[derive(Deserialize, Debug, DeriveIntoActiveModel)]
pub struct CreateMarketRequest {
    pub name: String,
    pub start_date: NaiveDate,
    pub finish_date: NaiveDate,
}
#[derive(Deserialize, Debug, DeriveIntoActiveModel)]
pub struct UpdateMarketRequest {
    pub id: Uuid,
    pub name: String,
    pub start_date: NaiveDate,
    pub finish_date: NaiveDate,
}
#[derive(Deserialize, Debug, DeriveIntoActiveModel)]
pub struct UpdateMarketStatusRequest {
    pub id: Uuid,
    pub status: MarketStatus,
}
