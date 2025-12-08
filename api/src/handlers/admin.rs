use crate::{
    AppState,
    error::GlobalError,
    models::admin::InvitationPayload,
    utils::{auth::Claims, common::CustomMessage},
};
use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use entity::invitation::Entity as Invitation;
use entity::users::Entity as User;
use entity::{invitation, users};
use macros::has_role;
use migration::OnConflict;
use redis::AsyncTypedCommands;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, TransactionTrait,
};

pub fn admin_router() -> Router<AppState> {
    Router::new().route("/invitations", post(send_invitations))
}

#[has_role(Admin)]
async fn send_invitations(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<InvitationPayload>,
) -> impl IntoResponse {
    let mut redis_con = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(GlobalError::RedisErr)?;

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

    let invitation_models = payload.emails.iter().map(|email| invitation::ActiveModel {
        email: Set(email.to_owned()),
        roles: Set(payload.roles.iter().map(|r| r.to_string()).collect()),
        ..Default::default()
    });

    let txn = state.conn.begin().await.map_err(GlobalError::DbErr)?;
    Invitation::insert_many(invitation_models)
        .on_conflict(
            OnConflict::column(invitation::Column::Email)
                .update_columns([invitation::Column::Roles, invitation::Column::DateExpired])
                .to_owned(),
        )
        .exec(&txn)
        .await
        .map_err(GlobalError::DbErr)?;

    let inserted_invitations = Invitation::find()
        .filter(invitation::Column::Email.is_in(payload.emails.clone()))
        .all(&txn)
        .await
        .map_err(GlobalError::DbErr)?;

    txn.commit().await.map_err(GlobalError::DbErr)?;

    for invitation in &inserted_invitations {
        let _ = redis_con
            .xadd(
                "invitations_stream",
                "*",
                &[
                    ("id", &invitation.id.to_string()),
                    ("receiver", &invitation.email),
                    ("sender_first_name", &claims.first_name),
                    ("sender_last_name", &claims.last_name),
                ],
            )
            .await
            .map_err(GlobalError::RedisErr)?;
    }

    Ok(Json(CustomMessage {
        message: format!(
            "Приглашения успешно отправлены в кол-ве {}",
            inserted_invitations.len()
        ),
    }))
}
