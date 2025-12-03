use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}
};
use chrono::{Duration, Utc};
use entity::users;
use entity::users::Entity as User;
use jsonwebtoken::{Header, encode};
use sea_orm::ActiveModelTrait;
use serde_json::json;
use crate::{AppState, error::GlobalError, models::auth::{AuthResponse, LoginPayload, ProtectedResponse, RegisterPayload, Role}, utils::{Claims, KEYS, hash_password, verify_password}};

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/registration-test", post(registration_test))
        .route("/protected", get(protected))
}

fn generate_token(sub: String) -> Result<String, GlobalError> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::hours(24)).timestamp() as usize;

    let claims = Claims {
        sub,
        iat,
        exp,
    };

    return encode(&Header::default(), &claims, &KEYS.encoding).map_err(|_| GlobalError::TokenCreation);
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>
    ) -> impl IntoResponse {

    let user: users::Model = User::find_by_email(payload.email)
        .one(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?
        .ok_or(GlobalError::NotFound)?;

    if !verify_password(&user.password, &payload.password) {
        return Err(GlobalError::WrongCredentials);
    }
    let token = generate_token(user.id.to_owned().to_string())?;

    Ok(Json(AuthResponse { token }))
}

async fn registration_test(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>
    ) -> Result<impl IntoResponse, GlobalError>  {

    let roles: Vec<String> = vec![
        Role::Initiator,
        Role::Admin,
        Role::ProjectOffice,
        Role::Expert,
    ]
    .iter()
    .map(|r| r.to_string())
    .collect();

    let mut user = users::ActiveModel::from_json(json!(payload)).map_err(|_| GlobalError::BadRequest)?;

    user.set(users::Column::Password, hash_password(&payload.password)?.into());
    user.set(users::Column::Roles, roles.into());

    let user: users::Model= user.insert(&state.conn).await.map_err(GlobalError::DbErr)?;

    let token = generate_token(user.id.to_owned().to_string())?;

    Ok(Json(AuthResponse { token }))
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