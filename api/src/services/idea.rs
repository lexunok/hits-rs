use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QuerySelect, Set,
    TransactionTrait,
};
use uuid::Uuid;

use crate::{
    dtos::idea::{
        CreateIdeaRequest, IdeaResponse, IdeaSkillRequest, UpdateIdeaRequest,
        UpdateIdeaStatusRequest,
    },
    dtos::skill::SkillDto,
    error::AppError,
    services::{group::GroupService, profile::ProfileService},
    AppState,
};
use entity::{
    company, group,
    idea::{self, IdeaStatus},
    idea_checked, idea_skill, skill, users,
};

pub struct IdeaService;

impl IdeaService {
    async fn idea_model_to_response(
        state: &AppState,
        model: idea::Model,
    ) -> Result<IdeaResponse, AppError> {
        let initiator = ProfileService::get_user_dto_by_id(state, model.initiator_id).await?;

        let experts = if let Some(group_id) = model.group_expert_id {
            Some(GroupService::get_one(state, group_id).await?)
        } else {
            None
        };

        let project_office = if let Some(group_id) = model.group_project_office_id {
            Some(GroupService::get_one(state, group_id).await?)
        } else {
            None
        };

        Ok(IdeaResponse {
            id: model.id,
            initiator,
            name: model.name,
            experts,
            project_office,
            is_checked: false, // TODO: Implement is_checked
            status: model.status,
            created_at: model.created_at,
            modified_at: model.modified_at,
            is_active: model.is_active,
            problem: model.problem,
            solution: model.solution,
            result: model.result,
            customer: model.customer,
            contact_person: model.contact_person,
            description: model.description,
            suitability: model.suitability,
            budget: model.budget,
            pre_assessment: model.pre_assessment,
            rating: model.rating,
            max_team_size: model.max_team_size,
            min_team_size: model.min_team_size,
        })
    }

    pub async fn get_idea(
        state: &AppState,
        idea_id: Uuid,
        user_id: Uuid,
    ) -> Result<IdeaResponse, AppError> {
        let idea = idea::Entity::find_by_id(idea_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Idea not found".to_owned()))?;

        let is_checked = idea_checked::Entity::find()
            .filter(
                idea_checked::Column::IdeaId
                    .eq(idea_id)
                    .and(idea_checked::Column::UserId.eq(user_id)),
            )
            .one(&state.db)
            .await?
            .is_some();

        if !is_checked {
            let checked = idea_checked::ActiveModel {
                idea_id: Set(idea_id),
                user_id: Set(user_id),
            };
            checked.insert(&state.db).await?;
        }

        let mut response = Self::idea_model_to_response(state, idea).await?;
        response.is_checked = true;
        Ok(response)
    }

    pub async fn get_all(state: &AppState) -> Result<Vec<IdeaResponse>, AppError> {
        let ideas = idea::Entity::find().all(&state.db).await?;
        let mut responses = Vec::new();
        for idea in ideas {
            responses.push(Self::idea_model_to_response(state, idea).await?);
        }
        Ok(responses)
    }

    pub async fn get_list_by_initiator(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Vec<IdeaResponse>, AppError> {
        let ideas = idea::Entity::find()
            .filter(idea::Column::InitiatorId.eq(user_id))
            .all(&state.db)
            .await?;
        let mut responses = Vec::new();
        for idea in ideas {
            responses.push(Self::idea_model_to_response(state, idea).await?);
        }
        Ok(responses)
    }

    pub async fn create(
        state: &AppState,
        payload: CreateIdeaRequest,
        initiator_id: Uuid,
    ) -> Result<IdeaResponse, AppError> {
        let idea = idea::ActiveModel {
            initiator_id: Set(initiator_id),
            name: Set(payload.name),
            group_expert_id: Set(payload.group_expert_id),
            group_project_office_id: Set(payload.group_project_office_id),
            status: Set(IdeaStatus::New),
            problem: Set(payload.problem),
            solution: Set(payload.solution),
            result: Set(payload.result),
            customer: Set(payload.customer),
            contact_person: Set(payload.contact_person),
            description: Set(payload.description),
            suitability: Set(payload.suitability),
            budget: Set(payload.budget),
            max_team_size: Set(payload.max_team_size),
            min_team_size: Set(payload.min_team_size),
            ..Default::default()
        };

        let idea = idea.insert(&state.db).await?;
        Self::idea_model_to_response(state, idea).await
    }

    pub async fn update_by_initiator(
        state: &AppState,
        payload: UpdateIdeaRequest,
        initiator_id: Uuid,
    ) -> Result<IdeaResponse, AppError> {
        let idea = idea::Entity::find_by_id(payload.id)
            .filter(idea::Column::InitiatorId.eq(initiator_id))
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Idea not found or you don't have access".to_owned()))?;

        let mut idea: idea::ActiveModel = idea.into();
        idea.name = Set(payload.name);
        idea.group_expert_id = Set(payload.group_expert_id);
        idea.group_project_office_id = Set(payload.group_project_office_id);
        idea.problem = Set(payload.problem);
        idea.solution = Set(payload.solution);
        idea.result = Set(payload.result);
        idea.customer = Set(payload.customer);
        idea.contact_person = Set(payload.contact_person);
        idea.description = Set(payload.description);
        idea.suitability = Set(payload.suitability);
        idea.budget = Set(payload.budget);
        idea.max_team_size = Set(payload.max_team_size);
        idea.min_team_size = Set(payload.min_team_size);

        let idea = idea.update(&state.db).await?;
        Self::idea_model_to_response(state, idea).await
    }

    pub async fn update_by_admin(
        state: &AppState,
        payload: UpdateIdeaRequest,
    ) -> Result<IdeaResponse, AppError> {
        let idea = idea::Entity::find_by_id(payload.id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Idea not found".to_owned()))?;

        let mut idea: idea::ActiveModel = idea.into();
        idea.name = Set(payload.name);
        idea.group_expert_id = Set(payload.group_expert_id);
        idea.group_project_office_id = Set(payload.group_project_office_id);
        idea.problem = Set(payload.problem);
        idea.solution = Set(payload.solution);
        idea.result = Set(payload.result);
        idea.customer = Set(payload.customer);
        idea.contact_person = Set(payload.contact_person);
        idea.description = Set(payload.description);
        idea.suitability = Set(payload.suitability);
        idea.budget = Set(payload.budget);
        idea.max_team_size = Set(payload.max_team_size);
        idea.min_team_size = Set(payload.min_team_size);

        let idea = idea.update(&state.db).await?;
        Self::idea_model_to_response(state, idea).await
    }

    pub async fn delete_by_initiator(
        state: &AppState,
        id: Uuid,
        initiator_id: Uuid,
    ) -> Result<(), AppError> {
        let res = idea::Entity::delete_by_id(id)
            .filter(idea::Column::InitiatorId.eq(initiator_id))
            .exec(&state.db)
            .await?;

        if res.rows_affected == 0 {
            return Err(AppError::NotFound(
                "Idea not found or you don't have access".to_owned(),
            ));
        }
        Ok(())
    }

    pub async fn delete_by_admin(state: &AppState, id: Uuid) -> Result<(), AppError> {
        let res = idea::Entity::delete_by_id(id).exec(&state.db).await?;
        if res.rows_affected == 0 {
            return Err(AppError::NotFound("Idea not found".to_owned()));
        }
        Ok(())
    }

    pub async fn update_status_by_initiator(
        state: &AppState,
        id: Uuid,
        initiator_id: Uuid,
    ) -> Result<(), AppError> {
        let idea = idea::Entity::find_by_id(id)
            .filter(idea::Column::InitiatorId.eq(initiator_id))
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Idea not found or you don't have access".to_owned()))?;

        let mut idea: idea::ActiveModel = idea.into();
        idea.status = Set(IdeaStatus::OnApproval);
        idea.update(&state.db).await?;
        Ok(())
    }

    pub async fn update_status(
        state: &AppState,
        id: Uuid,
        status: UpdateIdeaStatusRequest,
    ) -> Result<(), AppError> {
        let idea = idea::Entity::find_by_id(id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Idea not found".to_owned()))?;

        let mut idea: idea::ActiveModel = idea.into();
        idea.status = Set(status.status);
        idea.update(&state.db).await?;
        Ok(())
    }

    pub async fn get_idea_skills(
        state: &AppState,
        idea_id: Uuid,
    ) -> Result<Vec<SkillDto>, AppError> {
        let idea = idea::Entity::find_by_id(idea_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Idea not found".to_owned()))?;

        let skills = idea
            .find_related(skill::Entity)
            .all(&state.db)
            .await?
            .into_iter()
            .map(|model| SkillDto::from(model))
            .collect();

        Ok(skills)
    }

    pub async fn update_idea_skills(
        state: &AppState,
        payload: IdeaSkillRequest,
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<(), AppError> {
        let idea = idea::Entity::find_by_id(payload.idea_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Idea not found".to_owned()))?;

        if idea.initiator_id != user_id && !is_admin {
            return Err(AppError::Forbidden);
        }

        let skills_to_add: Vec<idea_skill::ActiveModel> = payload
            .skills
            .iter()
            .map(|skill| idea_skill::ActiveModel {
                idea_id: Set(payload.idea_id),
                skill_id: Set(skill.id),
            })
            .collect();

        state.db.transaction::<_, (), AppError>(|txn| {
            Box::pin(async move {
                idea_skill::Entity::delete_many()
                    .filter(idea_skill::Column::IdeaId.eq(payload.idea_id))
                    .exec(txn)
                    .await?;

                if !skills_to_add.is_empty() {
                    idea_skill::Entity::insert_many(skills_to_add)
                        .exec(txn)
                        .await?;
                }

                Ok(())
            })
        }).await?;

        Ok(())
    }
}
