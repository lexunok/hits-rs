use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
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
    RedisErr(redis::RedisError),
    Forbidden,
    Custom(String),
}
impl IntoResponse for GlobalError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            GlobalError::WrongCredentials => {
                (StatusCode::UNAUTHORIZED, "Wrong credentials".to_string())
            }
            GlobalError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error".to_string(),
            ),
            GlobalError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".to_string()),
            GlobalError::NotFound => (StatusCode::NOT_FOUND, "Not Found".to_string()),
            GlobalError::BadRequest => (StatusCode::BAD_REQUEST, "Bad Request".to_string()),
            GlobalError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid Server Error".to_string(),
            ),
            GlobalError::DbErr(e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            GlobalError::RedisErr(e) => {
                tracing::error!("Redis error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Redis error".to_string())
            }
            GlobalError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            GlobalError::Custom(s) => (StatusCode::BAD_REQUEST, s),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
