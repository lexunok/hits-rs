use crate::{AppState, dtos::tag::{CreateTagRequest, TagDto, UpdateTagRequest}, error::AppError};
use entity::{prelude::Tag, tag};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, IntoActiveModel, prelude::Uuid};

pub struct TagService;

impl TagService {
    pub async fn get_all(state: &AppState) -> Vec<TagDto> {
        Tag::find()
            .into_partial_model()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }

    pub async fn create_confirmed(
        state: &AppState,
        payload: CreateTagRequest,
        creator_id: Uuid,
    ) -> Result<TagDto, AppError> {
        let tag = tag::ActiveModel {
            name: Set(payload.name),
            color: Set(payload.color),
            confirmed: Set(true),
            creator_id: Set(Some(creator_id)),
            ..Default::default()
        }
        .insert(&state.conn)
        .await?;

        Ok(TagDto {
            id: tag.id,
            name: tag.name,
            color: tag.color,
            confirmed: tag.confirmed,
            creator_id: tag.creator_id,
            updater_id: tag.updater_id,
            deleter_id: tag.deleter_id,
        })
    }

    pub async fn create_unconfirmed(
        state: &AppState,
        payload: CreateTagRequest,
        creator_id: Uuid,
    ) -> Result<TagDto, AppError> {
        let tag = tag::ActiveModel {
            name: Set(payload.name),
            color: Set(payload.color),
            confirmed: Set(false),
            creator_id: Set(Some(creator_id)),
            ..Default::default()
        }
        .insert(&state.conn)
        .await?;

        Ok(TagDto {
            id: tag.id,
            name: tag.name,
            color: tag.color,
            confirmed: tag.confirmed,
            creator_id: tag.creator_id,
            updater_id: tag.updater_id,
            deleter_id: tag.deleter_id,
        })
    }

    pub async fn confirm(
        state: &AppState,
        tag_id: Uuid,
        updater_id: Uuid,
    ) -> Result<(), AppError> {
        let mut tag = Tag::find_by_id(tag_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        tag.confirmed = Set(true);
        tag.updater_id = Set(Some(updater_id));
        tag.update(&state.conn).await?;

        Ok(())
    }

    pub async fn update(
        state: &AppState,
        tag_id: Uuid,
        payload: UpdateTagRequest,
        updater_id: Uuid,
    ) -> Result<(), AppError> {
        let mut tag = Tag::find_by_id(tag_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        tag.name = Set(payload.name);
        tag.color = Set(payload.color);
        tag.updater_id = Set(Some(updater_id));
        tag.update(&state.conn).await?;

        Ok(())
    }

    pub async fn delete(state: &AppState, tag_id: Uuid) -> Result<(), AppError> {
        Tag::delete_by_id(tag_id).exec(&state.conn).await?;
        Ok(())
    }
}
