use macros::IntoDataResponse;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: u64,
    pub page_size: u64,
}
#[derive(IntoDataResponse, Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}
