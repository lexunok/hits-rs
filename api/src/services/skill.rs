use crate::{
    AppState,
    dtos::skill::{CreateSkillRequest, SkillDto, UpdateSkillRequest},
    error::AppError,
};
use entity::{prelude::*, skill, skill_type::SkillType};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter,
    prelude::Uuid,
};
use std::collections::HashMap;

pub struct SkillService;

impl SkillService {
    pub async fn get_all(state: &AppState) -> Result<Vec<SkillDto>, AppError> {
        let skills = Skill::find()
            .filter(skill::Column::DeletedAt.is_null())
            .into_partial_model::<SkillDto>()
            .all(&state.conn)
            .await?;
        Ok(skills)
    }

    pub async fn get_all_confirmed_or_creator(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<HashMap<SkillType, Vec<SkillDto>>, AppError> {
        let skills: Vec<SkillDto> = Skill::find()
            .filter(
                Condition::all()
                    .add(skill::Column::DeletedAt.is_null())
                    .add(
                        Condition::any()
                            .add(skill::Column::Confirmed.eq(true))
                            .add(skill::Column::CreatorId.eq(user_id)),
                    ),
            )
            .into_partial_model::<SkillDto>()
            .all(&state.conn)
            .await?;

        let mut map = HashMap::new();
        for skill in skills {
            map.entry(skill.skill_type.clone()).or_default().push(skill);
        }
        Ok(map)
    }

    pub async fn get_by_type(
        state: &AppState,
        skill_type: SkillType,
    ) -> Result<Vec<SkillDto>, AppError> {
        let skills = Skill::find()
            .filter(
                Condition::all()
                    .add(skill::Column::DeletedAt.is_null())
                    .add(skill::Column::SkillType.eq(skill_type)),
            )
            .into_partial_model::<SkillDto>()
            .all(&state.conn)
            .await?;
        Ok(skills)
    }

    pub async fn create(
        state: &AppState,
        payload: CreateSkillRequest,
        creator_id: Uuid,
        is_confirmed: bool,
    ) -> Result<SkillDto, AppError> {
        let new_skill = skill::ActiveModel {
            name: Set(payload.name),
            skill_type: Set(payload.skill_type),
            creator_id: Set(creator_id),
            confirmed: Set(is_confirmed),
            ..Default::default()
        };

        let skill = new_skill.insert(&state.conn).await?;
        let skill_dto = Skill::find_by_id(skill.id)
            .into_partial_model::<SkillDto>()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(skill_dto)
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

        skill.updater_id = Set(Some(updater_id));
        skill.updated_at = Set(Some(chrono::Utc::now().into()));

        let updated_skill = skill.update(&state.conn).await?;

        let skill_dto = Skill::find_by_id(updated_skill.id)
            .into_partial_model::<SkillDto>()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(skill_dto)
    }

    pub async fn confirm(
        state: &AppState,
        skill_id: Uuid,
        updater_id: Uuid,
    ) -> Result<SkillDto, AppError> {
        let mut skill = Skill::find_by_id(skill_id)
            .filter(skill::Column::DeletedAt.is_null())
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        skill.confirmed = Set(true);
        skill.updater_id = Set(Some(updater_id));
        skill.updated_at = Set(Some(chrono::utc::Utc::now().into()));

        let updated_skill = skill.update(&state.conn).await?;
        let skill_dto = Skill::find_by_id(updated_skill.id)
            .into_partial_model::<SkillDto>()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(skill_dto)
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

        skill.deleted_at = Set(Some(chrono::Utc::now().into()));
        skill.deleter_id = Set(Some(deleter_id));

        skill.update(&state.conn).await?;
        Ok(())
    }
}
