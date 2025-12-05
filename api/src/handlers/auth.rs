use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}
};
use axum_extra::extract::CookieJar;
use entity::users;
use entity::users::Entity as User;
use jsonwebtoken::{Validation, decode};
use sea_orm::ActiveModelTrait;
use serde_json::json;
use crate::{AppState, error::GlobalError, models::auth::{LoginPayload, ProtectedResponse, RegisterPayload, Role}, utils::auth::{Claims, KEYS, TokenType, generate_tokens, hash_password, verify_password}};
use macros::has_role;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/registration-test", post(registration_test))
        .route("/protected", get(protected))
        .route("/refresh", post(refresh))
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
    
    generate_tokens(user.id.to_string(), user.roles)
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

    generate_tokens(user.id.to_string(), user.roles)
}

pub async fn refresh(
    jar: CookieJar
) -> Result<impl IntoResponse, GlobalError> {

    let refresh_cookie = jar
        .get("refresh_token")
        .ok_or(GlobalError::WrongCredentials)?;
    
    let refresh_token = refresh_cookie.value();

    let token_data = decode::<Claims>(
        refresh_token,
        &KEYS.decoding,
        &Validation::default()
    ).map_err(|_| GlobalError::InvalidToken)?;

    if token_data.claims.token_type != TokenType::Refresh {
        return Err(GlobalError::InvalidToken);
    }

    generate_tokens(token_data.claims.sub, token_data.claims.roles)
}

#[has_role(Admin)]
async fn protected(claims: Claims) -> Result<impl IntoResponse, GlobalError> {
    Ok((
        StatusCode::OK,
        Json(ProtectedResponse {
            message: "Welcome to the protected area!".to_string(),
            user_id: claims.sub,
        }),
    ))
}