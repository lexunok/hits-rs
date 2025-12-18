use crate::{
    AppState,
    dtos::{
        common::MessageResponse, invitation::{InvitationPayload, InvitationResponse},
    },
    error::AppError,
    services::invitation::InvitationService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use macros::has_role;
use sea_orm::prelude::Uuid;

pub fn invitation_router() -> Router<AppState> {
    Router::new()
        .route("/", post(send_invitations))
        .route("/{id}", get(get_invitation))
}

async fn get_invitation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<InvitationResponse, AppError> {
    let invitation = InvitationService::get_invitation(&state, id).await?;

    Ok(InvitationResponse {
        email: invitation.email,
        code: invitation.id,
    })
}

#[has_role(Admin)]
async fn send_invitations(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<InvitationPayload>,
) -> Result<MessageResponse, AppError> {
    let sent_count = InvitationService::send_invitations(&state, claims, payload).await?;

    if sent_count == 0 {
        return Ok(MessageResponse {
            message: "Все приглашения по указанным email уже были отправлены ранее.".to_string(),
        });
    }

    Ok(MessageResponse {
        message: format!(
            "Новые приглашения успешно отправлены в кол-ве {}",
            sent_count
        ),
    })
}
