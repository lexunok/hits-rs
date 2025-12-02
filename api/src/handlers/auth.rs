use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Header, encode};
use crate::{models::auth::{LoginPayload, LoginResponse, ProtectedResponse}, utils::{AuthError,Claims,KEYS}};

pub fn auth_router() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/protected", get(protected))
}

async fn login(Json(payload): Json<LoginPayload>) -> impl IntoResponse {

    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    if payload.username != "lexunok" || payload.password != "2505" {
        return Err(AuthError::WrongCredentials);
    }

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::hours(24)).timestamp() as usize;

    let claims = Claims {
        sub: "userId".to_owned(),
        iat,
        exp,
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(LoginResponse { token }))
}

async fn protected(claims: Claims) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ProtectedResponse {
            message: "Welcome to the protected area!".to_string(),
            user_id: claims.sub,
        }),
    )
}