use crate::{
    AppState,
    config::GLOBAL_CONFIG,
    error::GlobalError,
    models::{
        auth::{InvitationResponse, LoginPayload, PasswordResetPayload, RegisterPayload},
        common::{CustomMessage, IdResponse, ParamsId},
    },
    utils::{
        auth::{Claims, TokenType, generate_tokens, hash_password, verify_password},
        smtp::send_code_to_update_password,
    },
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post, put},
};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Local};
use entity::{
    invitation::{self, Entity as Invitation},
    users::{self, Entity as User},
    verification_code::{self, Entity as VerificationCode},
};
use jsonwebtoken::{Validation, decode};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait,
    prelude::{Expr, Uuid},
};
use serde_json::json;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/invitation/:id", get(get_invitation))
        .route("/login", post(login))
        .route("/registration", post(registration))
        .route("/refresh", post(refresh))
        .route(
            "/password/verification/:email",
            post(request_to_update_password),
        )
        .route("/password/:id", put(confirm_and_update_password))
}

async fn get_invitation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, GlobalError> {
    let mut invitation: invitation::ActiveModel = Invitation::find_by_id(id)
        .filter(invitation::Column::ExpiryDate.gt(Local::now()))
        .one(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?
        .ok_or(GlobalError::NotFound)?
        .into_active_model();

    invitation.set(
        invitation::Column::ExpiryDate,
        (Local::now() + Duration::hours(3)).into(),
    );

    let invitation: invitation::Model = invitation
        .update(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?;

    Ok(Json(InvitationResponse {
        email: invitation.email,
        code: invitation.id,
    }))
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
        user.email,
        user.first_name,
        user.last_name,
        user.roles,
    )
}

async fn registration(
    Query(params): Query<ParamsId>,
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, GlobalError> {
    let txn = state.conn.begin().await.map_err(GlobalError::DbErr)?;

    let invitation: invitation::Model = Invitation::find_by_id(params.id)
        .filter(invitation::Column::ExpiryDate.gt(Local::now()))
        .one(&txn)
        .await
        .map_err(GlobalError::DbErr)?
        .ok_or(GlobalError::NotFound)?;

    let mut user =
        users::ActiveModel::from_json(json!(payload)).map_err(|_| GlobalError::BadRequest)?;

    user.set(users::Column::Email, invitation.email.to_lowercase().into());
    user.set(
        users::Column::Password,
        hash_password(&payload.password)?.into(),
    );
    user.set(users::Column::Roles, invitation.roles.clone().into());

    let user: users::Model = user.insert(&txn).await.map_err(GlobalError::DbErr)?;

    let mut invitation: invitation::ActiveModel = invitation.into();

    invitation.set(invitation::Column::ExpiryDate, Local::now().into());

    invitation.update(&txn).await.map_err(GlobalError::DbErr)?;

    txn.commit().await.map_err(GlobalError::DbErr)?;

    generate_tokens(
        user.id.to_string(),
        user.email,
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

    let token_data = decode::<Claims>(
        refresh_token,
        &GLOBAL_CONFIG.decoding_key,
        &Validation::default(),
    )
    .map_err(|_| GlobalError::InvalidToken)?;

    if token_data.claims.token_type != TokenType::Refresh {
        return Err(GlobalError::InvalidToken);
    }

    generate_tokens(
        token_data.claims.sub,
        token_data.claims.email,
        token_data.claims.first_name,
        token_data.claims.last_name,
        token_data.claims.roles,
    )
}

async fn request_to_update_password(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> Result<impl IntoResponse, GlobalError> {
    User::find_by_email(email.to_lowercase())
        .one(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?
        .ok_or(GlobalError::NotFound)?;

    let mut rng = OsRng;
    let random_u32 = rng.next_u32();
    let code = (100_000 + (random_u32 % 900_000)).to_string();

    let txn = state.conn.begin().await.map_err(GlobalError::DbErr)?;

    VerificationCode::update_many()
        .col_expr(
            verification_code::Column::ExpiryDate,
            Expr::value(Local::now().naive_local()),
        )
        .filter(verification_code::Column::Email.eq(email.to_lowercase().clone()))
        .filter(verification_code::Column::ExpiryDate.gt(Local::now()))
        .exec(&txn)
        .await
        .map_err(GlobalError::DbErr)?;

    let verification_code = verification_code::ActiveModel {
        email: Set(email.to_lowercase()),
        code: Set(hash_password(&code)?),
        expiry_date: Set((Local::now() + Duration::minutes(10)).into()),
        ..Default::default()
    };

    let verification_code: verification_code::Model = verification_code
        .insert(&txn)
        .await
        .map_err(GlobalError::DbErr)?;

    send_code_to_update_password(code, email)
        .await
        .map_err(|e| GlobalError::Custom(e.to_string()))?;

    txn.commit().await.map_err(GlobalError::DbErr)?;

    Ok(Json(IdResponse {
        id: verification_code.id,
    }))
}

async fn confirm_and_update_password(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetPayload>,
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

        let mut user: users::ActiveModel =
            User::find_by_email(verification_code.email.to_lowercase().clone())
                .one(&txn)
                .await
                .map_err(GlobalError::DbErr)?
                .ok_or(GlobalError::NotFound)?
                .into_active_model();

        user.set(
            users::Column::Password,
            hash_password(&payload.password)?.into(),
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
        message: "Успешное обновление пароля".to_string(),
    }))
}
