use crate::{
    AppState,
    dtos::{group::GroupDto, profile::UserDto, skill::{CreateSkillRequest, SkillDto, UpdateSkillRequest}},
    error::AppError,
};
use chrono::Local;
use entity::{prelude::*, skill, skill_type::SkillType, users};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, EntityLoaderTrait, LoaderTrait, IntoActiveModel, QueryFilter, prelude::Uuid
};
use std::collections::HashMap;

pub struct GroupService;

impl GroupService {
    pub async fn get_all(state: &AppState) -> Vec<GroupDto> {
        Group::find()
            .into_partial_model()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }

    pub async fn get_one(
        state: &AppState,
        id: Uuid,
    ) -> Result<GroupDto, AppError> {
        let group = Group::find_by_id(id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_ex();

        let users: Vec<UserDto> = group.users.iter().map(|u| UserDto{
            id: u.id,
            study_group: u.study_group.to_owned(),
            telephone: u.telephone.to_owned(),
            roles: u.roles.to_owned(),
            email: u.email.to_owned(),
            last_name: u.last_name.to_owned(),
            first_name: u.first_name.to_owned(),
            created_at: u.created_at.into()
        }).collect();

        Ok(GroupDto {
            id: group.id,
            name: group.name,
            roles: group.roles,
            users
        })
    }

    pub async fn create(
        state: &AppState,
        payload: CreateSkillRequest,
        creator_id: Uuid,
        is_confirmed: bool,
    ) -> Result<SkillDto, AppError> {
        Skill::find()                                                                                                  
            .filter(skill::Column::Name.eq(&payload.name))             
            .filter(skill::Column::SkillType.eq(payload.skill_type.clone()))                                                                                                                                                                                             
            .filter(skill::Column::DeletedAt.is_null())                                                                                                        
            .one(&state.conn)                                                                                                                                  
            .await
            .map_err(|_| AppError::Custom("Навык с таким именем и типом уже существует.".to_string()))?;                                                                                                                                           

        let new_skill = skill::ActiveModel {
            name: Set(payload.name),
            skill_type: Set(payload.skill_type),
            creator_id: Set(creator_id),
            confirmed: Set(is_confirmed),
            ..Default::default()
        };

        let skill = new_skill.insert(&state.conn).await?;
        Ok(SkillDto { 
            id: skill.id, 
            name: skill.name, 
            skill_type: skill.skill_type, 
            confirmed: skill.confirmed, 
            creator_id: skill.creator_id, 
            updater_id: skill.updater_id, 
            deleter_id: skill.deleter_id 
        })
    }

    pub async fn update(
        state: &AppState,
        payload: UpdateSkillRequest,
        updater_id: Uuid,
    ) -> Result<SkillDto, AppError> {
        let mut skill = Skill::find_by_id(payload.id)
            .filter(skill::Column::DeletedAt.is_null())
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        if let Some(name) = payload.name {
            skill.name = Set(name);
        }
        if let Some(skill_type) = payload.skill_type {
            skill.skill_type = Set(skill_type);
        }
        if let Some(confirmed) = payload.confirmed {
            skill.confirmed = Set(confirmed);
        }

        skill.updater_id = Set(Some(updater_id));
        skill.updated_at = Set(Some(Local::now().into()));

        let skill = skill.update(&state.conn).await?;

        Ok(SkillDto { 
            id: skill.id, 
            name: skill.name, 
            skill_type: skill.skill_type, 
            confirmed: skill.confirmed, 
            creator_id: skill.creator_id, 
            updater_id: skill.updater_id, 
            deleter_id: skill.deleter_id 
        })
    }

    pub async fn delete(
        state: &AppState,
        skill_id: Uuid,
        deleter_id: Uuid,
    ) -> Result<(), AppError> {
        let mut skill = Skill::find_by_id(skill_id)
            .filter(skill::Column::DeletedAt.is_null())
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        skill.deleted_at = Set(Some(Local::now().into()));
        skill.deleter_id = Set(Some(deleter_id));

        skill.update(&state.conn).await?;
        Ok(())
    }
}
