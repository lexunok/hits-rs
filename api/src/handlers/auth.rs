use crate::{
    AppState,
    error::GlobalError,
    models::{auth::{InvitationResponse, LoginPayload, PasswordResetPayload, RegisterPayload}, common::{CustomMessage, IdResponse, ParamsId}},
    utils::{auth::{Claims, KEYS, TokenType, generate_tokens, hash_password, verify_password}, smtp::send_code_to_reset_password},
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::{Json, Router, extract::{Path, Query, State}, response::IntoResponse, routing::{get, post}};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Local};
use entity::{invitation::{self, Entity as Invitation}, password_reset::{self, Entity as PasswordReset}};
use entity::users;
use entity::users::Entity as User;
use jsonwebtoken::{Validation, decode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait, prelude::Uuid};
use serde_json::json;
use sea_orm::prelude::Expr;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/invitation/:id", get(get_invitation))
        .route("/login", post(login))
        .route("/registration", post(registration))
        .route("/refresh", post(refresh))
        .route("/recovery-password/:email", post(recovery_password))
        .route("/new-password/:id", post(new_password))
}

async fn get_invitation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>
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

    let invitation: invitation::Model = invitation.update(&state.conn).await.map_err(GlobalError::DbErr)?;

    Ok(Json(InvitationResponse{
        email: invitation.email,
        code: invitation.id
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
        invitation::Column::ExpiryDate,
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

async fn recovery_password(
    State(state): State<AppState>,
    Path(email): Path<String>
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

    PasswordReset::update_many()
        .col_expr(
            password_reset::Column::ExpiryDate, 
            Expr::value(Local::now().naive_local())
        )
        .filter(password_reset::Column::Email.eq(email.to_lowercase().clone()))
        .filter(password_reset::Column::ExpiryDate.gt(Local::now()))
        .exec(&txn)
        .await
        .map_err(GlobalError::DbErr)?;

    let password_reset = password_reset::ActiveModel {
            email: Set(email.to_lowercase()),
            code: Set(hash_password(&code)?),
            expiry_date: Set((Local::now() + Duration::minutes(10)).into()),
            ..Default::default()
    };

    let password_reset: password_reset::Model = password_reset.insert(&txn).await.map_err(GlobalError::DbErr)?;
    
    send_code_to_reset_password(code, email).await.map_err(|e| GlobalError::Custom(e.to_string()))?;

    txn.commit().await.map_err(GlobalError::DbErr)?;

    Ok(Json(IdResponse{id: password_reset.id}))
}

async fn new_password(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetPayload>,
) -> Result<impl IntoResponse, GlobalError> {
    let password_reset: password_reset::Model = PasswordReset::find_by_id(payload.id)
        .one(&state.conn)
        .await
        .map_err(GlobalError::DbErr)?
        .ok_or(GlobalError::NotFound)?;

        
    if Local::now() > password_reset.expiry_date {
        return Err(GlobalError::Custom("Время запроса истекло".to_string()));
    }
    if password_reset.wrong_tries >= 3 {
        return Err(GlobalError::Custom("Превышено максимальное количество попыток".to_string()));
    }

    if verify_password(&password_reset.code, &payload.code) {
        let txn = state.conn.begin().await.map_err(GlobalError::DbErr)?;

        let mut user: users::ActiveModel = User::find_by_email(password_reset.email.to_lowercase().clone())
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

        PasswordReset::update_many()
            .col_expr(
                password_reset::Column::ExpiryDate, 
                Expr::value(Local::now().naive_local())
            )
            .filter(password_reset::Column::Email.eq(password_reset.email.to_lowercase()))
            .filter(password_reset::Column::ExpiryDate.gt(Local::now()))
            .exec(&txn)
            .await
            .map_err(GlobalError::DbErr)?;

        txn.commit().await.map_err(GlobalError::DbErr)?;
    } else {
        let wrong_tries = password_reset.wrong_tries + 1;
        let mut password_reset = password_reset.into_active_model();
        
        password_reset.set(
            password_reset::Column::WrongTries,
            wrong_tries.into(),
        );
        
        password_reset.update(&state.conn).await.map_err(GlobalError::DbErr)?;

        return Err(GlobalError::Custom("Ошибка, попробуйте еще раз".to_string()));
    }

    Ok(Json(CustomMessage{
        message: "Успешное обновление пароля".to_string()
    }))
}