use crate::{
    AppState,
    error::GlobalError,
    models::{
        admin::{InvitationPayload, RegisterPayload},
        common::CustomMessage,
    },
    utils::auth::{Claims, hash_password},
    workers::invitation_worker::INVITATIONS_STREAM_NAME,
};
use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use chrono::Local;
use entity::{
    invitation::{self, Entity as Invitation},
    users::{self, Entity as User},
};
use macros::has_role;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QuerySelect,
};
use serde_json::json;

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
) -> impl IntoResponse {
    let mut redis_con = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(GlobalError::RedisErr)?;

    let existing_users: Vec<String> = User::find()
        .select_only()
        .column(users::Column::Email)
        .filter(users::Column::Email.is_in(payload.emails.clone()))
        .into_tuple()
        .all(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?;

    if !existing_users.is_empty() {
        return Err(GlobalError::Custom(format!(
            "Следующие email уже зарегистрированы: {}",
            existing_users.join(", ")
        )));
    }

    let existing_invitation_emails: Vec<String> = Invitation::find()
        .select_only()
        .column(invitation::Column::Email)
        .filter(invitation::Column::Email.is_in(payload.emails.clone()))
        .filter(invitation::Column::ExpiryDate.gt(Local::now()))
        .into_tuple()
        .all(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?
        .into_iter()
        .collect();

    let new_emails: Vec<String> = payload
        .emails
        .into_iter()
        .filter(|email| !existing_invitation_emails.contains(email))
        .collect();

    if new_emails.is_empty() {
        return Ok(Json(CustomMessage {
            message: "Все приглашения по указанным email уже были отправлены ранее.".to_string(),
        }));
    }

    let invitation_models = new_emails.iter().map(|email| invitation::ActiveModel {
        email: Set(email.to_owned()),
        roles: Set(payload.roles.iter().map(|r| r.to_string()).collect()),
        ..Default::default()
    });

    let inserted_invitations = Invitation::insert_many(invitation_models)
        .exec_with_returning(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?;

    let mut redis_pipe = redis::pipe();
    for invitation in &inserted_invitations {
        redis_pipe.xadd(
            INVITATIONS_STREAM_NAME,
            "*",
            &[
                ("id", &invitation.id.to_string()),
                ("receiver", &invitation.email),
                ("sender_first_name", &claims.first_name),
                ("sender_last_name", &claims.last_name),
            ],
        );
    }
    let _: () = redis_pipe
        .query_async(&mut redis_con)
        .await
        .map_err(GlobalError::RedisErr)?;

    Ok(Json(CustomMessage {
        message: format!(
            "Новые приглашения успешно отправлены в кол-ве {}",
            inserted_invitations.len()
        ),
    }))
}

#[has_role(Admin)]
async fn registration(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, GlobalError> {
    let mut user =
        users::ActiveModel::from_json(json!(payload)).map_err(|_| GlobalError::BadRequest)?;

    user.set(
        users::Column::Password,
        hash_password(&payload.password)?.into(),
    );

    user.insert(&state.conn).await.map_err(GlobalError::DbErr)?;

    Ok(Json(CustomMessage {
        message: "Пользователь успешно создан".to_string(),
    }))
}
