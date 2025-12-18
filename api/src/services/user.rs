use crate::{
    AppState,
    dtos::{
        common::PaginationParams,
        profile::UserDto,
        user::{UserCreatePayload, UserUpdatePayload},
    },
    error::AppError,
    utils::security::hash_password,
};
use entity::users::{self, Entity as User};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder, prelude::Uuid,
};
use serde_json::json;

pub struct UserService;

impl UserService {
    pub async fn get_all(state: &AppState, pagination: PaginationParams) -> Vec<UserDto> {
        User::find()
            .filter(users::Column::IsDeleted.eq(false))
            .order_by_desc(users::Column::CreatedAt)
            .into_partial_model()
            .paginate(&state.conn, pagination.page_size)
            .fetch_page(pagination.page)
            .await
            .unwrap_or_default()
    }
    pub async fn get_one(state: &AppState, id: Uuid) -> Result<UserDto, AppError> {
        User::find_by_id(id)
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)
    }
    pub async fn create(state: &AppState, payload: UserCreatePayload) -> Result<(), AppError> {
        let mut user =
            users::ActiveModel::from_json(json!(payload)).map_err(|_| AppError::BadRequest)?;

        user.email = Set(payload.email.to_lowercase());
        user.password = Set(hash_password(&payload.password)?);

        user.insert(&state.conn).await?;

        Ok(())
    }
    pub async fn update(state: &AppState, payload: UserUpdatePayload) -> Result<(), AppError> {
        let mut user =
            users::ActiveModel::from_json(json!(payload)).map_err(|_| AppError::BadRequest)?;

        user.email = Set(payload.email.to_lowercase());

        user.update(&state.conn).await?;

        Ok(())
    }
    pub async fn restore(state: &AppState, email: String) -> Result<(), AppError> {
        let mut user = User::find_by_email(email)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        user.is_deleted = Set(false);

        user.update(&state.conn).await?;

        Ok(())
    }
    pub async fn delete(state: &AppState, id: Uuid) -> Result<(), AppError> {
        let mut user = User::find_by_id(id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        user.is_deleted = Set(true);

        user.update(&state.conn).await?;

        Ok(())
    }
}
