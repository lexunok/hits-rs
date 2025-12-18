use crate::{
    AppState, dtos::invitation::InvitationPayload, error::AppError, utils::security::Claims,
    workers::invitation_worker::INVITATIONS_STREAM_NAME,
};
use chrono::{Duration, Local};
use entity::{
    invitation::{self, Entity as Invitation},
    users::{self, Entity as User},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QuerySelect, prelude::Uuid,
};

pub struct InvitationService;

impl InvitationService {
    pub async fn get_invitation(state: &AppState, id: Uuid) -> Result<invitation::Model, AppError> {
        let mut invitation = Invitation::find_by_id(id)
            .filter(invitation::Column::ExpiryDate.gt(Local::now()))
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();
        invitation.expiry_date = Set((Local::now() + Duration::hours(3)).into());

        Ok(invitation.update(&state.conn).await?)
    }
    pub async fn send_invitations(
        state: &AppState,
        claims: Claims,
        payload: InvitationPayload,
    ) -> Result<usize, AppError> {
        let mut redis_con = state
            .redis_client
            .get_multiplexed_async_connection()
            .await?;

        let existing_users: Vec<String> = User::find()
            .select_only()
            .column(users::Column::Email)
            .filter(users::Column::Email.is_in(payload.emails.clone()))
            .into_tuple()
            .all(&state.conn)
            .await?;

        if !existing_users.is_empty() {
            return Err(AppError::Custom(format!(
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
            .await?
            .into_iter()
            .collect();

        let new_emails: Vec<String> = payload
            .emails
            .into_iter()
            .filter(|email| !existing_invitation_emails.contains(email))
            .collect();

        if new_emails.is_empty() {
            return Ok(0);
        }

        let invitation_models = new_emails.iter().map(|email| invitation::ActiveModel {
            email: Set(email.to_owned()),
            roles: Set(payload.roles.clone()),
            ..Default::default()
        });

        let inserted_invitations = Invitation::insert_many(invitation_models)
            .exec_with_returning(&state.conn)
            .await?;

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
        let _: () = redis_pipe.query_async(&mut redis_con).await?;

        Ok(inserted_invitations.len())
    }
}
