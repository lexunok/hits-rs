use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}
};
use axum_extra::extract::CookieJar;
use entity::{invitation, users};
use entity::users::Entity as User;
use entity::invitation::Entity as Invitation;
use jsonwebtoken::{Validation, decode};
use migration::OnConflict;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, QueryFilter, QuerySelect};
use serde_json::json;
use crate::{AppState, error::GlobalError, models::{admin::InvitationPayload, auth::{LoginPayload, ProtectedResponse, RegisterPayload, Role}}, utils::{auth::{Claims, KEYS, TokenType, generate_tokens, hash_password, verify_password}, common::CustomMessage}};
use macros::has_role;
use sea_orm::EntityTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryTrait;
use migration::ConnectionTrait;

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/invitation", post(send_invitation))
}

#[has_role(Admin)]
async fn send_invitation(
    State(state): State<AppState>,
    Json(payload): Json<InvitationPayload>,
    claims: Claims
    ) -> impl IntoResponse {

    let existing: Vec<String> = User::find()
        .select_only()
        .column(users::Column::Email)
        .filter(users::Column::Email.is_in(payload.email.clone()))
        .into_tuple()
        .all(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?;

    if !existing.is_empty() {
        return Err(GlobalError::Custom(format!(
            "Следующие email уже зарегистрированы: {}",
            existing.join(", ")
        )));
    }

    let invitation_models = payload.email.iter().map(|email| {
        invitation::ActiveModel {
            email: Set(email.to_owned()),
            roles: Set(payload.roles.iter().map(|r| r.to_string()).collect()),
            ..Default::default()
        }
    });

    let res = Invitation::insert_many(invitation_models)
        .on_conflict(
            OnConflict::column(invitation::Column::Email)
                .update_columns([invitation::Column::Roles, invitation::Column::DateExpired])
                .to_owned()
        )
        .exec_with_returning_keys(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?;

    Ok(Json(CustomMessage{message: format!("Приглашения успешно отправлены в кол-ве {}", res.len())}))
}