use crate::{
    AppState,
    dtos::skill::{CreateSkillRequest, SkillDto, UpdateSkillRequest},
    error::AppError,
};
use chrono::Local;
use entity::{prelude::*, skill, skill_type::SkillType};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, ExprTrait, IntoActiveModel, QueryFilter, prelude::Uuid
};
use std::collections::HashMap;

pub struct SkillService;

impl SkillService {
    pub async fn get_all(state: &AppState) -> Vec<SkillDto> {
        Skill::find()
            .filter(skill::Column::DeletedAt.is_null())
            .into_partial_model::<SkillDto>()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }

    pub async fn get_all_my_or_confirmed(
        state: &AppState,
        user_id: Uuid,
    ) -> HashMap<String, Vec<SkillDto>> {
        let skills: Vec<SkillDto> = Skill::find()
            .filter(skill::Column::Confirmed.eq(true).or(skill::Column::CreatorId.eq(user_id)))
            .filter(skill::Column::DeletedAt.is_null())
            .into_partial_model()
            .all(&state.conn)
            .await
            .unwrap_or_default();

        let mut map: HashMap<String, Vec<SkillDto>>  = HashMap::new();
        for skill in skills {
            map.entry(skill.skill_type.to_string()).or_default().push(skill);
        }
        map
    }

    pub async fn get_by_type(
        state: &AppState,
        skill_type: SkillType,
    ) -> Vec<SkillDto> {
        Skill::find()
            .filter(skill::Column::SkillType.eq(skill_type))
            .filter(skill::Column::DeletedAt.is_null())
            .into_partial_model::<SkillDto>()
            .all(&state.conn)
            .await
            .unwrap_or_default()
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
