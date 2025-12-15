use crate::{
    AppState,
    dtos::{
        auth::EmailResetPayload,
        common::{ApiMessageResponse, IdResponse},
    },
    error::AppError,
    services::user::UserService,
    utils::security::Claims,
};
use axum::{
    Router,
    extract::{Path, State},
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
) -> Result<IdResponse, AppError> {
    let verification_id = UserService::request_email_change(&state, new_email).await?;

    Ok(IdResponse {
        id: verification_id,
    })
}

async fn confirm_and_update_email(
    State(state): State<AppState>,
    claims: Claims,
    payload: axum::Json<EmailResetPayload>,
) -> Result<ApiMessageResponse, AppError> {
    UserService::confirm_email_change(&state, claims, payload.0).await?;

    Ok(ApiMessageResponse {
        success: true,
        message: "Успешное обновление почты".to_string(),
    })
}
