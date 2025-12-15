use crate::{
    AppState,
    dtos::{
        admin::{InvitationPayload, RegisterPayload},
        common::ApiMessageResponse,
    },
    error::AppError,
    services::{invitation::InvitationService, user::UserService},
    utils::security::Claims,
};
use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use macros::has_role;

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/invitations", post(send_invitations))
        .route("/registration", post(registration))
}

#[has_role(Admin)]
async fn send_invitations(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<InvitationPayload>,
) -> Result<ApiMessageResponse, AppError> {
    let sent_count = InvitationService::send_invitations(&state, claims, payload).await?;

    if sent_count == 0 {
        return Ok(ApiMessageResponse {
            success: true,
            message: "Все приглашения по указанным email уже были отправлены ранее.".to_string(),
        });
    }

    Ok(ApiMessageResponse {
        success: true,
        message: format!(
            "Новые приглашения успешно отправлены в кол-ве {}",
            sent_count
        ),
    })
}

#[has_role(Admin)]
async fn registration(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    UserService::create_user_by_admin(&state, payload).await?;

    Ok(ApiMessageResponse {
        success: true,
        message: "Пользователь успешно создан".to_string(),
    })
}
