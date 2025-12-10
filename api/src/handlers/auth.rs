use crate::{
    AppState,
    error::GlobalError,
    models::auth::{LoginPayload, RegisterPayload},
    utils::auth::{Claims, KEYS, TokenType, generate_tokens, hash_password, verify_password},
};
use axum::{Json, Router, extract::{Query, State}, response::IntoResponse, routing::post};
use axum_extra::extract::CookieJar;
use chrono::Local;
use entity::invitation::{self, Entity as Invitation};
use entity::users;
use entity::users::Entity as User;
use jsonwebtoken::{Validation, decode};
use sea_orm::{ActiveModelTrait,ColumnTrait, TransactionTrait, QueryFilter, EntityTrait, prelude::Uuid};
use serde::Deserialize;
use serde_json::json;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/registration", post(registration))
        .route("/refresh", post(refresh))
}

#[derive(Deserialize)]
struct Params {
    code: Uuid,
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let user: users::Model = User::find_by_email(payload.email.to_lowercase())
        .one(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?
        .ok_or(GlobalError::NotFound)?;

    if !verify_password(&user.password, &payload.password) {
        return Err(GlobalError::WrongCredentials);
    }

    generate_tokens(
        user.id.to_string(),
        user.first_name,
        user.last_name,
        user.roles,
    )
}

async fn registration(
    Query(params): Query<Params>,
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, GlobalError> {

    let txn = state.conn.begin().await.map_err(GlobalError::DbErr)?;

    let invitation: invitation::Model = Invitation::find_by_id(params.code)
        .filter(invitation::Column::DateExpired.gt(Local::now()))
        .one(&txn)
        .await
        .map_err(GlobalError::DbErr)?
        .ok_or(GlobalError::NotFound)?;

    let mut user =
        users::ActiveModel::from_json(json!(payload)).map_err(|_| GlobalError::BadRequest)?;

    user.set(
        users::Column::Email,
        invitation.email.to_lowercase().into(),
    );
    user.set(
        users::Column::Password,
        hash_password(&payload.password)?.into(),
    );
    user.set(
        users::Column::Roles,
        invitation.roles.clone().into(),
    );

    let user: users::Model = user.insert(&txn).await.map_err(GlobalError::DbErr)?;

    let mut invitation: invitation::ActiveModel = invitation.into();

    invitation.set(
        invitation::Column::DateExpired,
        Local::now().into(),
    );

    invitation.update(&txn).await.map_err(GlobalError::DbErr)?;

    txn.commit().await.map_err(GlobalError::DbErr)?;

    generate_tokens(
        user.id.to_string(),
        user.first_name,
        user.last_name,
        user.roles,
    )
}

pub async fn refresh(jar: CookieJar) -> Result<impl IntoResponse, GlobalError> {
    let refresh_cookie = jar
        .get("refresh_token")
        .ok_or(GlobalError::WrongCredentials)?;

    let refresh_token = refresh_cookie.value();

    let token_data = decode::<Claims>(refresh_token, &KEYS.decoding, &Validation::default())
        .map_err(|_| GlobalError::InvalidToken)?;

    if token_data.claims.token_type != TokenType::Refresh {
        return Err(GlobalError::InvalidToken);
    }

    generate_tokens(
        token_data.claims.sub,
        token_data.claims.first_name,
        token_data.claims.last_name,
        token_data.claims.roles,
    )
}
