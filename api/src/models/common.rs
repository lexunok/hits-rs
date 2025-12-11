use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CustomMessage {
    pub message: String,
}
#[derive(Debug, Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}
#[derive(Deserialize)]
pub struct ParamsId {
    pub id: Uuid,
}