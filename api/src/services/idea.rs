use crate::{
    AppState,
    dtos::{
        group::GroupDto,
        idea::{
            IdeaDto, IdeaQueryResult, IdeaSkillRequest, IdeaStatusRequest, IdeaWithChecked,
            SaveIdeaRequest,
        },
        skill::SkillDto,
    },
    error::AppError,
};
use chrono::Local;
use entity::{
    group,
    idea::{self, Entity as Idea},
    idea_checked, idea_skill,
    idea_status::IdeaStatus,
    prelude::{Group, IdeaSkill},
    skill, users,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, ExprTrait, IntoActiveModel, QueryFilter,
    QuerySelect, RelationTrait, Set, TransactionTrait,
    prelude::Uuid,
    sea_query::{Expr, Query},
};
use validator::Validate;

pub struct IdeaService;

impl IdeaService {
    pub async fn get_one(
        state: &AppState,
        idea_id: Uuid,
        user_id: Uuid,
    ) -> Result<IdeaWithChecked, AppError> {
        let mut idea: IdeaQueryResult = Idea::find_by_id(idea_id)
            .join_as(
                sea_orm::JoinType::LeftJoin,
                idea::Relation::Experts.def(),
                "experts",
            )
            .join_as(
                sea_orm::JoinType::LeftJoin,
                idea::Relation::ProjectOffice.def(),
                "project_office",
            )
            .left_join(users::Entity)
            .column_as(users::Column::Email, "initiator_email")
            .column_as(users::Column::FirstName, "initiator_first_name")
            .column_as(users::Column::LastName, "initiator_last_name")
            .column_as(Expr::col(("experts", group::Column::Id)), "experts_id")
            .column_as(
                Expr::col(("project_office", group::Column::Id)),
                "project_office_id",
            )
            .column_as(Expr::col(("experts", group::Column::Name)), "experts_name")
            .column_as(
                Expr::col(("project_office", group::Column::Name)),
                "project_office_name",
            )
            .column_as(
                Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(idea_checked::Entity)
                        .and_where(Expr::col(idea_checked::Column::UserId).eq(user_id))
                        .and_where(
                            Expr::col(idea_checked::Column::IdeaId)
                                .equals((idea::Entity, idea::Column::Id)),
                        )
                        .to_owned(),
                ),
                "is_checked",
            )
            .into_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        if !idea.is_checked {
            let checked = idea_checked::ActiveModel {
                idea_id: Set(idea_id),
                user_id: Set(user_id),
            };
            let _ = checked.insert(&state.conn).await;
            idea.is_checked = true;
        }

        Ok(idea.into())
    }

    pub async fn get_all(
        state: &AppState,
        user_id: Uuid,
        initiator_filter: Option<Expr>,
    ) -> Vec<IdeaWithChecked> {
        Idea::find()
            .left_join(users::Entity)
            .column_as(users::Column::Email, "initiator_email")
            .column_as(users::Column::FirstName, "initiator_first_name")
            .column_as(users::Column::LastName, "initiator_last_name")
            .column_as(
                Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(idea_checked::Entity)
                        .and_where(Expr::col(idea_checked::Column::UserId).eq(user_id))
                        .and_where(
                            Expr::col(idea_checked::Column::IdeaId)
                                .equals((idea::Entity, idea::Column::Id)),
                        )
                        .to_owned(),
                ),
                "is_checked",
            )
            .filter(Condition::all().add_option(initiator_filter))
            .into_model::<IdeaQueryResult>()
            .all(&state.conn)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(IdeaWithChecked::from)
            .collect()
    }

    pub async fn get_all_by_initiator(state: &AppState, user_id: Uuid) -> Vec<IdeaWithChecked> {
        Self::get_all(state, user_id, Some(idea::Column::InitiatorId.eq(user_id))).await
    }
    pub async fn get_idea_skills(state: &AppState, id: Uuid) -> Vec<SkillDto> {
        IdeaSkill::find()
            .filter(idea_skill::Column::IdeaId.eq(id))
            .left_join(skill::Entity)
            .into_partial_model()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }

    pub async fn save_by_initiator(
        state: &AppState,
        payload: SaveIdeaRequest,
        initiator_id: Uuid,
    ) -> Result<IdeaDto, AppError> {
        Self::save(
            state,
            payload,
            initiator_id,
            Some(idea::Column::InitiatorId.eq(initiator_id)),
        )
        .await
    }
    pub async fn save(
        state: &AppState,
        payload: SaveIdeaRequest,
        initiator_id: Uuid,
        initiator_filter: Option<Expr>,
    ) -> Result<IdeaDto, AppError> {
        payload.validate()?;
        if payload.max_team_size < payload.min_team_size {
            return Err(AppError::Custom(
                "Максимальный размер команды должен быть больше минимального".to_string(),
            ));
        }

        let pre_assessment = (payload.suitability as f64 + payload.budget as f64) / 2.0;

        let mut idea = idea::ActiveModel {
            name: Set(payload.name),
            status: Set(payload.status),
            problem: Set(payload.problem),
            solution: Set(payload.solution),
            result: Set(payload.result),
            customer: Set(payload.customer),
            contact_person: Set(payload.contact_person),
            description: Set(payload.description),
            suitability: Set(payload.suitability),
            budget: Set(payload.budget),
            min_team_size: Set(payload.min_team_size),
            max_team_size: Set(payload.max_team_size),
            pre_assessment: Set(pre_assessment),
            rating: Set(0.0),
            is_active: Set(true),
            ..Default::default()
        };

        let mut idea_id: Uuid = payload.id.unwrap_or_default();

        if let Some(id) = payload.id {
            idea.id = Set(id);
            idea.modified_at = Set(Local::now().into());

            Idea::update(idea)
                .validate()?
                .filter(Condition::all().add_option(initiator_filter))
                .exec(&state.conn)
                .await?;
        } else {
            let experts: GroupDto = Group::find()
                .filter(Expr::cust(r#""group"."roles" @> ARRAY['EXPERT']"#))
                .into_partial_model()
                .one(&state.conn)
                .await?
                .ok_or(AppError::Custom(
                    "Не существует группы экспертов".to_string(),
                ))?;

            let project_office: GroupDto = Group::find()
                .filter(Expr::cust(r#""group"."roles" @> ARRAY['PROJECT_OFFICE']"#))
                .into_partial_model()
                .one(&state.conn)
                .await?
                .ok_or(AppError::Custom(
                    "Не существует группы проектного офиса".to_string(),
                ))?;
            idea.group_expert_id = Set(experts.id);
            idea.group_project_office_id = Set(project_office.id);
            idea.initiator_id = Set(initiator_id);

            idea_id = idea.insert(&state.conn).await?.id;

            // Нахожу список эскпертов в группе и для каждого эскперта делаю рейтинг запись
            // "INSERT INTO rating (expert_id, is_confirmed, idea_id) VALUES ('%s', FALSE, '%s');",
            // u.getUserId(), savedIdea.getId()
        }

        let response: IdeaDto = Idea::find_by_id(idea_id)
            .join(sea_orm::JoinType::LeftJoin, idea::Relation::Experts.def())
            .join_as(
                sea_orm::JoinType::LeftJoin,
                idea::Relation::ProjectOffice.def(),
                "project_office",
            )
            .left_join(users::Entity)
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        Ok(response)
    }
    pub async fn update_status_by_initiator(
        state: &AppState,
        id: Uuid,
        initiator_id: Uuid,
    ) -> Result<(), AppError> {
        let mut idea = Idea::find_by_id(id)
            .filter(idea::Column::InitiatorId.eq(initiator_id))
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        idea.status = Set(IdeaStatus::OnApproval);
        idea.modified_at = Set(Local::now().into());
        idea.update(&state.conn).await?;
        Ok(())
    }
    pub async fn update_status(
        state: &AppState,
        payload: IdeaStatusRequest,
    ) -> Result<(), AppError> {
        let mut idea = Idea::find_by_id(payload.id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        idea.status = Set(payload.status);
        idea.modified_at = Set(Local::now().into());
        idea.update(&state.conn).await?;
        Ok(())
    }
    pub async fn save_skills(
        state: &AppState,
        payload: IdeaSkillRequest,
        initiator_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(initiator_id) = initiator_id {
            Idea::find_by_id(payload.id)
                .filter(idea::Column::InitiatorId.eq(initiator_id))
                .one(&state.conn)
                .await?
                .ok_or(AppError::Forbidden)?;
        }

        let idea_skills = payload.skills.iter().map(|skill| idea_skill::ActiveModel {
            idea_id: Set(payload.id),
            skill_id: Set(skill.id),
        });

        let txn = state.conn.begin().await?;

        IdeaSkill::delete_many()
            .filter(idea_skill::Column::IdeaId.eq(payload.id))
            .exec(&txn)
            .await?;

        IdeaSkill::insert_many(idea_skills).exec(&txn).await?;

        txn.commit().await?;

        Ok(())
    }
    pub async fn delete(
        state: &AppState,
        idea_id: Uuid,
        initiator_filter: Option<Expr>,
    ) -> Result<(), AppError> {
        Idea::delete_by_id(idea_id)
            .filter(Condition::all().add_option(initiator_filter))
            .exec(&state.conn)
            .await?;
        Ok(())
    }
    pub async fn delete_by_initiator(
        state: &AppState,
        idea_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        Self::delete(&state, idea_id, Some(idea::Column::InitiatorId.eq(user_id))).await?;
        Ok(())
    }
}
