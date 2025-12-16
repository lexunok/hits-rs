use macros::IntoDataResponse;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ParamsId {
    pub id: Uuid,
}
#[derive(IntoDataResponse, Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}
