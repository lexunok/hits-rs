use crate::{
    AppState,
    config::GLOBAL_CONFIG,
    dtos::{
        user::{UserCreatePayload, UserUpdatePayload},
        auth::{EmailResetPayload, PasswordResetPayload},
        common::PaginationParams,
        profile::{ProfileUpdatePayload, UserDto},
    },
    error::AppError,
    utils::{
        security::{Claims, hash_password, verify_password},
        smtp::{send_code_to_update_email, send_code_to_update_password},
    },
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::body::Bytes;
use chrono::{Duration, Local};
use entity::{
    users::{self, Entity as User},
    verification_code::{self, Entity as VerificationCode},
};
use image::ImageFormat;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder,
    TransactionTrait,
    prelude::{Expr, Uuid},
};
use serde_json::json;
use std::path::PathBuf;
use validator::Validate;

pub struct UserService;

impl UserService {
    pub async fn get_users(state: &AppState, pagination: PaginationParams) -> Vec<UserDto> {
        User::find()
            .filter(users::Column::IsDeleted.eq(false))
            .order_by_desc(users::Column::CreatedAt)
            .into_partial_model()
            .paginate(&state.conn, pagination.page_size)
            .fetch_page(pagination.page)
            .await
            .unwrap_or_default()
    }
    pub async fn get_user(state: &AppState, id: Uuid) -> Result<UserDto, AppError> {
        User::find_by_id(id)
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)
    }
    pub async fn create_user(state: &AppState, payload: UserCreatePayload) -> Result<(), AppError> {
        let mut user =
            users::ActiveModel::from_json(json!(payload)).map_err(|_| AppError::BadRequest)?;

        user.email = Set(payload.email.to_lowercase());
        user.password = Set(hash_password(&payload.password)?);

        user.insert(&state.conn).await?;

        Ok(())
    }
    pub async fn update_user(state: &AppState, payload: UserUpdatePayload) -> Result<(), AppError> {
        let mut user =
            users::ActiveModel::from_json(json!(payload)).map_err(|_| AppError::BadRequest)?;

        user.email = Set(payload.email.to_lowercase());

        user.update(&state.conn).await?;

        Ok(())
    }
    pub async fn update_profile(
        state: &AppState,
        payload: ProfileUpdatePayload,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut user = User::find_by_id(id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        user.first_name = Set(payload.first_name);
        user.last_name = Set(payload.last_name);
        user.study_group = Set(payload.study_group);
        user.telephone = Set(payload.telephone);

        user.update(&state.conn).await?;

        Ok(())
    }
    pub async fn restore_user(state: &AppState, email: String) -> Result<(), AppError> {
        let mut user = User::find_by_email(email)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        user.is_deleted = Set(false);

        user.update(&state.conn).await?;

        Ok(())
    }
    pub async fn delete_user(state: &AppState, id: Uuid) -> Result<(), AppError> {
        let mut user = User::find_by_id(id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        user.is_deleted = Set(true);

        user.update(&state.conn).await?;

        Ok(())
    }

    pub async fn upload_avatar(
        user_id: Uuid,
        bytes: Bytes,
    ) -> Result<(), AppError> {
        let avatar_dir = PathBuf::from(&GLOBAL_CONFIG.avatar_path);
        let file_path = avatar_dir.join(format!("{}.webp", user_id));

        let img = image::load_from_memory(&bytes)
            .map_err(|_| AppError::Custom("Ошибка при загрузке изображения".to_string()))?;

        img.save_with_format(file_path, ImageFormat::WebP)
            .map_err(|_| AppError::Custom("Ошибка при сохранении аватара".to_string()))?;

        Ok(())
    }
    
    pub async fn confirm_email_change(
        state: &AppState,
        claims: Claims,
        payload: EmailResetPayload,
    ) -> Result<(), AppError> {
        payload.validate()?;
        Self::_verify_code(state, payload.id, payload.code, Some(claims.email), None).await
    }

    pub async fn confirm_password_reset(
        state: &AppState,
        payload: PasswordResetPayload,
    ) -> Result<(), AppError> {
        payload.validate()?;

        Self::_verify_code(
            state,
            payload.id,
            payload.code,
            None,
            Some(payload.password),
        )
        .await
    }

    pub async fn request_email_change(
        state: &AppState,
        new_email: String,
    ) -> Result<Uuid, AppError> {
        let user = User::find_by_email(new_email.to_lowercase())
            .one(&state.conn)
            .await?;

        if let Some(_) = user {
            return Err(AppError::Custom(
                "Пользователь с такой почтой уже существует!".to_string(),
            ));
        }

        let mut rng = OsRng;
        let random_u32 = rng.next_u32();
        let code = (100_000 + (random_u32 % 900_000)).to_string();

        let txn = state.conn.begin().await?;

        VerificationCode::update_many()
            .col_expr(
                verification_code::Column::ExpiryDate,
                Expr::value(Local::now()),
            )
            .filter(verification_code::Column::Email.eq(new_email.to_lowercase().clone()))
            .filter(verification_code::Column::ExpiryDate.gt(Local::now()))
            .exec(&txn)
            .await?;

        let verification_code = verification_code::ActiveModel {
            email: Set(new_email.to_lowercase()),
            code: Set(hash_password(&code)?),
            expiry_date: Set((Local::now() + Duration::minutes(10)).into()),
            ..Default::default()
        };

        let verification_code = verification_code.insert(&txn).await?;

        send_code_to_update_email(code, new_email)
            .await
            .map_err(|e| AppError::Custom(e.to_string()))?;

        txn.commit().await?;

        Ok(verification_code.id)
    }

    pub async fn request_password_reset(state: &AppState, email: String) -> Result<Uuid, AppError> {
        User::find_by_email(email.to_lowercase())
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut rng = OsRng;
        let random_u32 = rng.next_u32();
        let code = (100_000 + (random_u32 % 900_000)).to_string();

        let txn = state.conn.begin().await?;

        VerificationCode::update_many()
            .col_expr(
                verification_code::Column::ExpiryDate,
                Expr::value(Local::now()),
            )
            .filter(verification_code::Column::Email.eq(email.to_lowercase().clone()))
            .filter(verification_code::Column::ExpiryDate.gt(Local::now()))
            .exec(&txn)
            .await?;

        let verification_code = verification_code::ActiveModel {
            email: Set(email.to_lowercase()),
            code: Set(hash_password(&code)?),
            expiry_date: Set((Local::now() + Duration::minutes(10)).into()),
            ..Default::default()
        };

        let verification_code = verification_code.insert(&txn).await?;

        send_code_to_update_password(code, email)
            .await
            .map_err(|e| AppError::Custom(e.to_string()))?;

        txn.commit().await?;

        Ok(verification_code.id)
    }
    async fn _verify_code(
        state: &AppState,
        invitation_id: Uuid,
        code: String,
        email: Option<String>,
        password_data: Option<String>,
    ) -> Result<(), AppError> {
        let verification_code = VerificationCode::find_by_id(invitation_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        if Local::now() > verification_code.expiry_date {
            return Err(AppError::Custom("Время запроса истекло".to_string()));
        }
        if verification_code.wrong_tries >= 3 {
            return Err(AppError::Custom(
                "Превышено максимальное количество попыток".to_string(),
            ));
        }
        if verify_password(&verification_code.code, &code) {
            let txn = state.conn.begin().await?;

            let mut user = if let Some(email) = email {
                User::find_by_email(email)
            } else {
                User::find_by_email(verification_code.email.to_lowercase().clone())
            }
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

            if let Some(password_data) = password_data {
                user.password = Set(hash_password(&password_data)?);
            } else {
                user.email = Set(verification_code.email.to_lowercase().clone());
            }

            user.update(&txn).await?;

            VerificationCode::update_many()
                .col_expr(
                    verification_code::Column::ExpiryDate,
                    Expr::value(Local::now()),
                )
                .filter(verification_code::Column::Email.eq(verification_code.email.to_lowercase()))
                .filter(verification_code::Column::ExpiryDate.gt(Local::now()))
                .exec(&txn)
                .await?;

            txn.commit().await?;
        } else {
            let wrong_tries = verification_code.wrong_tries + 1;
            let mut verification_code = verification_code.into_active_model();

            verification_code.wrong_tries = Set(wrong_tries);

            verification_code.update(&state.conn).await?;

            return Err(AppError::Custom("Ошибка, попробуйте еще раз".to_string()));
        }
        Ok(())
    }
}
