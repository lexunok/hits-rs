use crate::{
    AppState,
    dtos::{
        auth::EmailResetPayload,
        common::{CustomMessage, IdResponse},
    },
    error::AppError,
    services::user::UserService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{post, put},
};

pub fn profile_router() -> Router<AppState> {
    Router::new()
        .route(
            "/email/verification/:new_email",
            post(request_to_update_email),
        )
        .route("/email/:id", put(confirm_and_update_email))
}

async fn request_to_update_email(
    State(state): State<AppState>,
    _: Claims,
    Path(new_email): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let verification_id = UserService::request_email_change(&state, new_email).await?;

    Ok(Json(IdResponse {
        id: verification_id,
    }))
}

async fn confirm_and_update_email(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<EmailResetPayload>,
) -> Result<impl IntoResponse, AppError> {
    UserService::confirm_email_change(&state, claims, payload).await?;

    Ok(Json(CustomMessage {
        message: "Успешное обновление почты".to_string(),
    }))
}
