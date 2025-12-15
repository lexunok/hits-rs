use axum::{
    Json,
    response::{IntoResponse, Response},
};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiDataResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct ApiMessageResponse {
    pub success: bool,
    pub message: String,
}

pub trait IntoApiResponse: Serialize {}

impl<T> IntoResponse for T
where
    T: IntoApiResponse,
{
    fn into_response(self) -> Response {
        let body = ApiDataResponse {
            success: true,
            data: Some(self),
        };
        Json(body).into_response()
    }
}

impl IntoResponse for ApiMessageResponse {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}
impl IntoApiResponse for IdResponse {}

#[derive(Deserialize)]
pub struct ParamsId {
    pub id: Uuid,
}
