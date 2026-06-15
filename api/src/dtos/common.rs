use macros::IntoDataResponse;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: u64,
    pub page_size: u64,
    pub search_text: Option<String>
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub count: u64,
    pub list: Vec<T>,
}

impl<T: Serialize> axum::response::IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}
