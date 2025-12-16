use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use macros::IntoDataResponse;

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct ParamsId {
    pub id: Uuid,
}
