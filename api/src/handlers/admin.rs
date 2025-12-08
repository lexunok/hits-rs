use axum::{
    Json, Router, extract::State, response::IntoResponse, routing::post
};
use entity::{invitation, users};
use entity::users::Entity as User;
use entity::invitation::Entity as Invitation;
use migration::OnConflict;
use sea_orm::{EntityTrait, ColumnTrait, ActiveValue::Set, QueryFilter, QuerySelect};
use crate::{AppState, error::GlobalError, models::admin::InvitationPayload, utils::{auth::Claims, common::CustomMessage}};
use macros::has_role;
use redis::AsyncTypedCommands;

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/invitations", post(send_invitations))
}

#[has_role(Admin)]
async fn send_invitations(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<InvitationPayload>,
    ) -> impl IntoResponse {

    let existing: Vec<String> = User::find()
        .select_only()
        .column(users::Column::Email)
        .filter(users::Column::Email.is_in(payload.emails.clone()))
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

    let invitation_models = payload.emails.iter().map(|email| {
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

    let mut redis_con = state.redis_client.get_multiplexed_async_connection().await.map_err(GlobalError::RedisErr)?;
    for email in &payload.emails {
        let _ = redis_con
            .xadd(
                "invitations_stream",
                "*",
                &[
                    ("link_id", &link_id),
                    ("receiver", &receiver),
                    ("sender_first_name", &sender_first_name),
                    ("sender_last_name", &sender_last_name),
                ],
            )
            .await
            .map_err(GlobalError::RedisErr)?;
    }

    Ok(Json(CustomMessage{message: format!("Приглашения успешно отправлены в кол-ве {}", res.len())}))
}