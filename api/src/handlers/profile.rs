use crate::{
    AppState,
    error::GlobalError,
    models::{
        auth::EmailResetPayload,
        common::{CustomMessage, IdResponse},
    },
    utils::{
        auth::{Claims, hash_password, verify_password},
        smtp::send_code_to_update_email,
    },
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{post, put},
};
use chrono::{Duration, Local};
use entity::{
    users::{self, Entity as User},
    verification_code::{self, Entity as VerificationCode},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    TransactionTrait, prelude::Expr,
};

pub fn profile_router() -> Router<AppState> {
    Router::new()
        .route(
            "/email/verification/:new_email",
            post(request_to_update_email),
        )
        .route("/email/:id", put(confirm_and_update_email))
}

async fn request_to_update_email(
    State(state): State<AppState>,
    _: Claims,
    Path(new_email): Path<String>,
) -> Result<impl IntoResponse, GlobalError> {
    let mut rng = OsRng;
    let random_u32 = rng.next_u32();
    let code = (100_000 + (random_u32 % 900_000)).to_string();

    let txn = state.conn.begin().await.map_err(GlobalError::DbErr)?;

    VerificationCode::update_many()
        .col_expr(
            verification_code::Column::ExpiryDate,
            Expr::value(Local::now().naive_local()),
        )
        .filter(verification_code::Column::Email.eq(new_email.to_lowercase().clone()))
        .filter(verification_code::Column::ExpiryDate.gt(Local::now()))
        .exec(&txn)
        .await
        .map_err(GlobalError::DbErr)?;

    let verification_code = verification_code::ActiveModel {
        email: Set(new_email.to_lowercase()),
        code: Set(hash_password(&code)?),
        expiry_date: Set((Local::now() + Duration::minutes(10)).into()),
        ..Default::default()
    };

    let verification_code: verification_code::Model = verification_code
        .insert(&txn)
        .await
        .map_err(GlobalError::DbErr)?;

    send_code_to_update_email(code, new_email)
        .await
        .map_err(|e| GlobalError::Custom(e.to_string()))?;

    txn.commit().await.map_err(GlobalError::DbErr)?;

    Ok(Json(IdResponse {
        id: verification_code.id,
    }))
}

async fn confirm_and_update_email(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<EmailResetPayload>,
) -> Result<impl IntoResponse, GlobalError> {
    let verification_code: verification_code::Model = VerificationCode::find_by_id(payload.id)
        .one(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?
        .ok_or(GlobalError::NotFound)?;

    if Local::now() > verification_code.expiry_date {
        return Err(GlobalError::Custom("Время запроса истекло".to_string()));
    }
    if verification_code.wrong_tries >= 3 {
        return Err(GlobalError::Custom(
            "Превышено максимальное количество попыток".to_string(),
        ));
    }

    if verify_password(&verification_code.code, &payload.code) {
        let txn = state.conn.begin().await.map_err(GlobalError::DbErr)?;

        let mut user: users::ActiveModel = User::find_by_email(claims.email)
            .one(&txn)
            .await
            .map_err(GlobalError::DbErr)?
            .ok_or(GlobalError::NotFound)?
            .into_active_model();

        user.set(
            users::Column::Email,
            verification_code.email.to_lowercase().clone().into(),
        );

        user.update(&txn).await.map_err(GlobalError::DbErr)?;

        VerificationCode::update_many()
            .col_expr(
                verification_code::Column::ExpiryDate,
                Expr::value(Local::now().naive_local()),
            )
            .filter(verification_code::Column::Email.eq(verification_code.email.to_lowercase()))
            .filter(verification_code::Column::ExpiryDate.gt(Local::now()))
            .exec(&txn)
            .await
            .map_err(GlobalError::DbErr)?;

        txn.commit().await.map_err(GlobalError::DbErr)?;
    } else {
        let wrong_tries = verification_code.wrong_tries + 1;
        let mut verification_code = verification_code.into_active_model();

        verification_code.set(verification_code::Column::WrongTries, wrong_tries.into());

        verification_code
            .update(&state.conn)
            .await
            .map_err(GlobalError::DbErr)?;

        return Err(GlobalError::Custom(
            "Ошибка, попробуйте еще раз".to_string(),
        ));
    }

    Ok(Json(CustomMessage {
        message: "Успешное обновление почты".to_string(),
    }))
}
