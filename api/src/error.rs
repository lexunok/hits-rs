use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json, 
};
use serde_json::json;

#[derive(Debug)]
pub enum GlobalError {
    WrongCredentials,
    TokenCreation,
    InvalidToken,
    NotFound,
    BadRequest,
    InternalServerError,
    DbErr(sea_orm::DbErr),
    Forbidden,
}
impl IntoResponse for GlobalError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            GlobalError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            GlobalError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            GlobalError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            GlobalError::NotFound => (StatusCode::NOT_FOUND, "Not Found"),
            GlobalError::BadRequest => (StatusCode::BAD_REQUEST, "Bad Request"),
            GlobalError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Invalid Server Error"),
            GlobalError::DbErr(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            },
            GlobalError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
