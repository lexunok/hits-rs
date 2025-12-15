use crate::{
    AppState,
    dtos::{
        auth::{InvitationResponse, LoginPayload, PasswordResetPayload, RegisterPayload},
        common::{CustomMessage, IdResponse, ParamsId},
    },
    error::AppError,
    services::{auth::AuthService, invitation::InvitationService, user::UserService},
    utils::security::generate_tokens,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post, put},
};
use axum_extra::extract::CookieJar;
use sea_orm::prelude::Uuid;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/invitation/:id", get(get_invitation))
        .route("/login", post(login))
        .route("/registration", post(registration))
        .route("/refresh", post(refresh))
        .route(
            "/password/verification/:email",
            post(request_to_update_password),
        )
        .route("/password/:id", put(confirm_and_update_password))
}

async fn get_invitation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let invitation = InvitationService::get_invitation(&state, id).await?;

    Ok(Json(InvitationResponse {
        email: invitation.email,
        code: invitation.id,
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthService::login(&state, payload).await?;

    generate_tokens(
        user.id.to_string(),
        user.email,
        user.first_name,
        user.last_name,
        user.roles,
    )
}

async fn registration(
    Query(params): Query<ParamsId>,
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthService::register_user(&state, params.id, payload).await?;

    generate_tokens(
        user.id.to_string(),
        user.email,
        user.first_name,
        user.last_name,
        user.roles,
    )
}

pub async fn refresh(jar: CookieJar) -> Result<impl IntoResponse, AppError> {
    let claims = AuthService::refresh(jar).await?;

    generate_tokens(
        claims.sub,
        claims.email,
        claims.first_name,
        claims.last_name,
        claims.roles,
    )
}

async fn request_to_update_password(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let verification_id = UserService::request_password_reset(&state, email).await?;

    Ok(Json(IdResponse {
        id: verification_id,
    }))
}

async fn confirm_and_update_password(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetPayload>,
) -> Result<impl IntoResponse, AppError> {
    UserService::confirm_password_reset(&state, payload).await?;

    Ok(Json(CustomMessage {
        message: "Успешное обновление пароля".to_string(),
    }))
}
