use std::collections::HashMap;

use chrono::Local;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, ExprTrait, IntoActiveModel, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set, TransactionTrait,
    prelude::Uuid,
    sea_query::{Expr, Query},
};

use crate::{
    AppState,
    dtos::{
        project::{
            AddToProjectRequest, ProjectBaseRow, ProjectDto, ProjectMarksDto, ProjectMarksRow,
            ProjectMemberDto, ProjectMemberRow, ProjectTaskRow, ProjectTeamDto, ReportProjectDto,
            TaskDto,
        },
        user::UserDto,
    },
    error::AppError,
    utils::query::load_tags_for_tasks,
};
use entity::{
    idea, idea_market, project, project_marks, project_member,
    project_role::ProjectRole,
    project_status::ProjectStatus,
    role::Role,
    sprint, sprint_status::SprintStatus,
    task, team, team_market_request, team_member, users,
};

pub struct ProjectService;

impl ProjectService {
    async fn get_base_projects(
        state: &AppState,
        filter: Option<Condition>,
    ) -> Result<Vec<ProjectBaseRow>, AppError> {
        Ok(entity::prelude::Project::find()
            .join(JoinType::InnerJoin, project::Relation::Idea.def())
            .join(JoinType::InnerJoin, idea::Relation::Users.def())
            .join(JoinType::InnerJoin, project::Relation::Team.def())
            .column_as(idea::Column::Name, "name")
            .column_as(idea::Column::Description, "description")
            .column_as(idea::Column::CompanyId, "company_id")
            .column_as(users::Column::Id, "initiator_id")
            .column_as(users::Column::Email, "initiator_email")
            .column_as(users::Column::FirstName, "initiator_first_name")
            .column_as(users::Column::LastName, "initiator_last_name")
            .column_as(team::Column::Id, "team_id")
            .column_as(team::Column::Name, "team_name")
            .column_as(
                Expr::expr(
                    Query::select()
                        .expr(Expr::val(1).count())
                        .from(entity::prelude::TeamMember)
                        .and_where(
                            Expr::col(team_member::Column::TeamId)
                                .equals((team::Entity, team::Column::Id)),
                        )
                        .and_where(Expr::col(team_member::Column::FinishDate).is_null())
                        .to_owned(),
                ),
                "team_members_count",
            )
            .filter(Condition::all().add_option(filter))
            .order_by_asc(project::Column::StartDate)
            .into_model()
            .all(&state.conn)
            .await?)
    }

    async fn get_members_map(
        state: &AppState,
        project_ids: &[Uuid],
        only_active: bool,
    ) -> Result<HashMap<Uuid, Vec<ProjectMemberDto>>, AppError> {
        if project_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut query = entity::prelude::ProjectMember::find()
            .filter(project_member::Column::ProjectId.is_in(project_ids.iter().copied()))
            .join(JoinType::InnerJoin, project_member::Relation::Users.def())
            .column_as(project_member::Column::ProjectId, "project_id")
            .column_as(users::Column::Email, "email")
            .column_as(users::Column::FirstName, "first_name")
            .column_as(users::Column::LastName, "last_name");
        if only_active {
            query = query.filter(project_member::Column::FinishDate.is_null());
        }
        let rows = query
            .into_model::<ProjectMemberRow>()
            .all(&state.conn)
            .await?;
        let mut result = HashMap::<Uuid, Vec<ProjectMemberDto>>::new();

        for row in rows {
            result
                .entry(row.project_id)
                .or_default()
                .push(ProjectMemberDto {
                    user_id: row.user_id,
                    team_id: row.team_id,
                    email: row.email,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    project_role: row.project_role,
                    start_date: row.start_date,
                    finish_date: row.finish_date,
                });
        }

        Ok(result)
    }

    async fn get_tasks_map(
        state: &AppState,
        project_ids: &[Uuid],
    ) -> Result<HashMap<(Uuid, Uuid), Vec<TaskDto>>, AppError> {
        if project_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let initiator = sea_orm::sea_query::Alias::new("initiator");
        let executor = sea_orm::sea_query::Alias::new("executor");

        let rows: Vec<ProjectTaskRow> = entity::prelude::Task::find()
            .filter(task::Column::ProjectId.is_in(project_ids.iter().copied()))
            .join_as(
                JoinType::LeftJoin,
                task::Relation::Users1.def(),
                initiator.clone(),
            )
            .join_as(
                JoinType::LeftJoin,
                task::Relation::Users2.def(),
                executor.clone(),
            )
            .tbl_col_as((initiator.clone(), users::Column::Id), "initiator_id")
            .tbl_col_as((initiator.clone(), users::Column::Email), "initiator_email")
            .tbl_col_as(
                (initiator.clone(), users::Column::FirstName),
                "initiator_first_name",
            )
            .tbl_col_as(
                (initiator.clone(), users::Column::LastName),
                "initiator_last_name",
            )
            .tbl_col_as((executor.clone(), users::Column::Id), "executor_id")
            .tbl_col_as((executor.clone(), users::Column::Email), "executor_email")
            .tbl_col_as(
                (executor.clone(), users::Column::FirstName),
                "executor_first_name",
            )
            .tbl_col_as(
                (executor.clone(), users::Column::LastName),
                "executor_last_name",
            )
            .order_by_asc(task::Column::Position)
            .into_model()
            .all(&state.conn)
            .await?;

        let task_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let mut tags_map = load_tags_for_tasks(state, &task_ids).await?;

        let mut task_map = HashMap::<Uuid, TaskDto>::new();
        let mut owner_map = HashMap::<Uuid, (Uuid, Uuid)>::new();

        for row in rows {
            let Some(executor_id) = row.executor_id else {
                continue;
            };

            owner_map.insert(row.id, (row.project_id, executor_id));

            task_map.entry(row.id).or_insert_with(|| TaskDto {
                id: row.id,
                sprint_id: row.sprint_id,
                project_id: row.project_id,
                position: row.position,
                name: row.name.clone(),
                description: row.description.clone(),
                leader_comment: row.leader_comment.clone(),
                executor_comment: row.executor_comment.clone(),
                initiator: UserDto::from_parts_opt(
                    row.initiator_id,
                    row.initiator_email.clone(),
                    row.initiator_first_name.clone(),
                    row.initiator_last_name.clone(),
                ),
                executor: UserDto::from_parts_opt(
                    row.executor_id,
                    row.executor_email.clone(),
                    row.executor_first_name.clone(),
                    row.executor_last_name.clone(),
                ),
                work_hour: row.work_hour,
                start_date: row.start_date,
                finish_date: row.finish_date,
                tags: tags_map.remove(&row.id).unwrap_or_default(),
                status: row.status,
            });
        }

        let mut result = HashMap::<(Uuid, Uuid), Vec<TaskDto>>::new();
        for (task_id, task) in task_map {
            if let Some(key) = owner_map.get(&task_id) {
                result.entry(*key).or_default().push(task);
            }
        }

        Ok(result)
    }

    async fn get_marks_map(
        state: &AppState,
        project_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<ProjectMarksDto>>, AppError> {
        if project_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows: Vec<ProjectMarksRow> = entity::prelude::ProjectMarks::find()
            .filter(project_marks::Column::ProjectId.is_in(project_ids.iter().copied()))
            .join(JoinType::InnerJoin, project_marks::Relation::Users.def())
            .column_as(project_marks::Column::ProjectId, "project_id")
            .column_as(project_marks::Column::UserId, "user_id")
            .column_as(users::Column::FirstName, "first_name")
            .column_as(users::Column::LastName, "last_name")
            .into_model()
            .all(&state.conn)
            .await?;

        let members_map = Self::get_members_map(state, project_ids, false).await?;
        let tasks_map = Self::get_tasks_map(state, project_ids).await?;
        let mut result = HashMap::<Uuid, Vec<ProjectMarksDto>>::new();

        for row in rows {
            let project_role = members_map
                .get(&row.project_id)
                .and_then(|members| members.iter().find(|member| member.user_id == row.user_id))
                .map(|member| member.project_role.clone())
                .unwrap_or(ProjectRole::Member);

            result.entry(row.project_id).or_default().push(ProjectMarksDto {
                project_id: row.project_id,
                user_id: row.user_id,
                first_name: row.first_name,
                last_name: row.last_name,
                project_role,
                mark: row.mark,
                tasks: tasks_map
                    .get(&(row.project_id, row.user_id))
                    .cloned()
                    .unwrap_or_default(),
            });
        }

        Ok(result)
    }

    async fn build_projects(
        state: &AppState,
        filter: Option<Condition>,
    ) -> Result<Vec<ProjectDto>, AppError> {
        let rows = Self::get_base_projects(state, filter).await?;
        let project_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let members_map = Self::get_members_map(state, &project_ids, false).await?;
        let marks_map = Self::get_marks_map(state, &project_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| ProjectDto {
                id: row.id,
                idea_id: row.idea_id,
                name: row.name,
                description: row.description,
                customer: row.customer,
                initiator: UserDto::from_parts(
                    row.initiator_id,
                    row.initiator_email,
                    row.initiator_first_name,
                    row.initiator_last_name,
                ),
                team: ProjectTeamDto {
                    id: row.team_id,
                    name: row.team_name,
                    members_count: row.team_members_count,
                },
                members: members_map.get(&row.id).cloned().unwrap_or_default(),
                report: ReportProjectDto {
                    project_id: row.id,
                    marks: marks_map.get(&row.id).cloned().unwrap_or_default(),
                    report: row.report,
                },
                start_date: row.start_date,
                finish_date: row.finish_date,
                status: row.status,
            })
            .collect())
    }

    pub async fn get_all(state: &AppState) -> Result<Vec<ProjectDto>, AppError> {
        Self::build_projects(state, None).await
    }

    pub async fn get_by_user(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Vec<ProjectDto>, AppError> {
        Self::build_projects(
            state,
            Some(
                Condition::all()
                    .add(project::Column::Status.ne(ProjectStatus::Deleted))
                    .add(
                        project::Column::Id.in_subquery(
                            Query::select()
                                .column(project_member::Column::ProjectId)
                                .from(project_member::Entity)
                                .and_where(project_member::Column::UserId.eq(user_id))
                                .to_owned(),
                        ),
                    ),
            ),
        )
        .await
    }

    pub async fn get_active_by_user(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Vec<ProjectDto>, AppError> {
        Self::build_projects(
            state,
            Some(
                Condition::all()
                    .add(project::Column::Status.eq(ProjectStatus::Active))
                    .add(
                        project::Column::Id.in_subquery(
                            Query::select()
                                .column(project_member::Column::ProjectId)
                                .from(project_member::Entity)
                                .and_where(project_member::Column::UserId.eq(user_id))
                                .to_owned(),
                        ),
                    ),
            ),
        )
        .await
    }

    pub async fn get_one(state: &AppState, project_id: Uuid) -> Result<ProjectDto, AppError> {
        Self::build_projects(
            state,
            Some(Condition::all().add(project::Column::Id.eq(project_id))),
        )
        .await?
        .into_iter()
        .next()
        .ok_or(AppError::NotFound)
    }

    pub async fn get_members(
        state: &AppState,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMemberDto>, AppError> {
        Ok(Self::get_members_map(state, &[project_id], true)
            .await?
            .remove(&project_id)
            .unwrap_or_default())
    }

    pub async fn get_marks(
        state: &AppState,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMarksDto>, AppError> {
        Ok(Self::get_marks_map(state, &[project_id])
            .await?
            .remove(&project_id)
            .unwrap_or_default())
    }

    pub async fn create_from_idea_market(
        state: &AppState,
        idea_market_id: Uuid,
    ) -> Result<ProjectDto, AppError> {
        let txn = state.conn.begin().await?;

        let idea_market = entity::prelude::IdeaMarket::find_by_id(idea_market_id)
            .join(JoinType::InnerJoin, idea_market::Relation::Market.def())
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let team_id = idea_market.team_id.ok_or(AppError::BadRequest)?;
        let idea = entity::prelude::Idea::find_by_id(idea_market.idea_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;
        let market = entity::prelude::Market::find_by_id(idea_market.market_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;
        let team = entity::prelude::Team::find_by_id(team_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let project = project::ActiveModel {
            idea_id: Set(idea_market.idea_id),
            team_id: Set(team_id),
            report: Set(None),
            start_date: Set(Local::now().date_naive()),
            finish_date: Set(Some(market.finish_date)),
            status: Set(ProjectStatus::Active),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        let team_members = entity::prelude::TeamMember::find()
            .filter(team_member::Column::TeamId.eq(team_id))
            .filter(team_member::Column::IsActive.eq(true))
            .all(&txn)
            .await?;

        for member in team_members {
            let role = if Some(member.user_id) == team.leader_id {
                ProjectRole::TeamLeader
            } else {
                ProjectRole::Member
            };

            project_member::ActiveModel {
                project_id: Set(project.id),
                user_id: Set(member.user_id),
                team_id: Set(Some(team_id)),
                project_role: Set(role),
                start_date: Set(Local::now().date_naive()),
                finish_date: Set(None),
            }
            .insert(&txn)
            .await?;
        }

        project_member::ActiveModel {
            project_id: Set(project.id),
            user_id: Set(idea.initiator_id),
            team_id: Set(None),
            project_role: Set(ProjectRole::Initiator),
            start_date: Set(Local::now().date_naive()),
            finish_date: Set(None),
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Self::get_one(state, project.id).await
    }

    pub async fn add_member(
        state: &AppState,
        project_id: Uuid,
        payload: AddToProjectRequest,
    ) -> Result<ProjectMemberDto, AppError> {
        let member = project_member::ActiveModel {
            project_id: Set(project_id),
            user_id: Set(payload.user_id),
            team_id: Set(payload.team_id),
            project_role: Set(ProjectRole::Member),
            start_date: Set(Local::now().date_naive()),
            finish_date: Set(None),
        }
        .insert(&state.conn)
        .await?;

        let user = entity::prelude::Users::find_by_id(member.user_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        Ok(ProjectMemberDto {
            user_id: member.user_id,
            team_id: member.team_id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            project_role: member.project_role,
            start_date: member.start_date,
            finish_date: member.finish_date,
        })
    }

    pub async fn kick_member_from_project_and_team(
        state: &AppState,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let project_member = entity::prelude::ProjectMember::find_by_id((project_id, user_id))
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        if matches!(
            project_member.project_role,
            ProjectRole::TeamLeader | ProjectRole::Initiator
        ) {
            return Err(AppError::Forbidden);
        }

        let today = Local::now().date_naive();
        let txn = state.conn.begin().await?;

        let mut member = project_member.into_active_model();
        let team_id = member.team_id.clone().take();
        member.finish_date = Set(Some(today));
        member.update(&txn).await?;

        if let Some(team_id) = team_id {
            entity::prelude::TeamMember::update_many()
                .col_expr(team_member::Column::IsActive, Expr::value(false))
                .col_expr(team_member::Column::FinishDate, Expr::value(Some(today)))
                .filter(team_member::Column::TeamId.eq(team_id))
                .filter(team_member::Column::UserId.eq(user_id))
                .exec(&txn)
                .await?;
        }

        txn.commit().await?;
        Ok(())
    }

    pub async fn pause(state: &AppState, project_id: Uuid) -> Result<(), AppError> {
        let mut project = entity::prelude::Project::find_by_id(project_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        project.status = Set(ProjectStatus::Paused);
        project.update(&state.conn).await?;
        Ok(())
    }

    pub async fn finish(
        state: &AppState,
        project_id: Uuid,
        report: String,
        user_id: Uuid,
        roles: &[Role],
    ) -> Result<(), AppError> {
        let txn = state.conn.begin().await?;

        let project = entity::prelude::Project::find_by_id(project_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;
        let idea = entity::prelude::Idea::find_by_id(project.idea_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;
        let team = entity::prelude::Team::find_by_id(project.team_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        if !roles.contains(&Role::Admin)
            && !roles.contains(&Role::ProjectOffice)
            && team.leader_id != Some(user_id)
            && idea.initiator_id != user_id
        {
            return Err(AppError::Forbidden);
        }

        let mut project = project.into_active_model();
        project.status = Set(ProjectStatus::Done);
        project.report = Set(Some(report));
        project.update(&txn).await?;

        let mut team = team.into_active_model();
        team.has_active_project = Set(false);
        team.update(&txn).await?;

        entity::prelude::ProjectMember::update_many()
            .col_expr(
                project_member::Column::FinishDate,
                Expr::value(Some(Local::now().date_naive())),
            )
            .filter(project_member::Column::ProjectId.eq(project_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;
        Ok(())
    }

    pub async fn change_team(
        state: &AppState,
        project_id: Uuid,
        new_team_id: Uuid,
    ) -> Result<(), AppError> {
        let today = Local::now().date_naive();
        let txn = state.conn.begin().await?;

        let new_team = entity::prelude::Team::find_by_id(new_team_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;
        if new_team.has_active_project {
            return Err(AppError::Custom("Команда занята".to_string()));
        }

        let active_sprint = entity::prelude::Sprint::find()
            .filter(sprint::Column::ProjectId.eq(project_id))
            .filter(sprint::Column::Status.eq(SprintStatus::Active))
            .one(&txn)
            .await?;
        if active_sprint.is_some() {
            return Err(AppError::Custom(
                "В проекте есть незавершенный спринт".to_string(),
            ));
        }

        let project = entity::prelude::Project::find_by_id(project_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;
        let old_team_id = project.team_id;

        entity::prelude::ProjectMember::update_many()
            .col_expr(project_member::Column::FinishDate, Expr::value(Some(today)))
            .filter(project_member::Column::ProjectId.eq(project_id))
            .filter(project_member::Column::FinishDate.is_null())
            .filter(project_member::Column::ProjectRole.ne(ProjectRole::Initiator))
            .exec(&txn)
            .await?;

        let new_team_members = entity::prelude::TeamMember::find()
            .filter(team_member::Column::TeamId.eq(new_team_id))
            .filter(team_member::Column::IsActive.eq(true))
            .all(&txn)
            .await?;

        for member in new_team_members {
            let role = if Some(member.user_id) == new_team.leader_id {
                ProjectRole::TeamLeader
            } else {
                ProjectRole::Member
            };

            project_member::ActiveModel {
                project_id: Set(project_id),
                user_id: Set(member.user_id),
                team_id: Set(Some(new_team_id)),
                project_role: Set(role),
                start_date: Set(today),
                finish_date: Set(None),
            }
            .insert(&txn)
            .await?;
        }

        entity::prelude::Team::update_many()
            .col_expr(team::Column::HasActiveProject, Expr::value(false))
            .filter(team::Column::Id.eq(old_team_id))
            .exec(&txn)
            .await?;

        entity::prelude::Team::update_many()
            .col_expr(team::Column::HasActiveProject, Expr::value(true))
            .filter(team::Column::Id.eq(new_team_id))
            .exec(&txn)
            .await?;

        entity::prelude::IdeaMarket::update_many()
            .col_expr(idea_market::Column::TeamId, Expr::value(Some(new_team_id)))
            .filter(idea_market::Column::IdeaId.eq(project.idea_id))
            .exec(&txn)
            .await?;

        entity::prelude::TeamMarketRequest::update_many()
            .col_expr(
                team_market_request::Column::Status,
                Expr::value(entity::request_status::RequestStatus::Annulled),
            )
            .filter(team_market_request::Column::TeamId.eq(old_team_id))
            .filter(
                team_market_request::Column::IdeaMarketId.in_subquery(
                    Query::select()
                        .column(idea_market::Column::Id)
                        .from(idea_market::Entity)
                        .and_where(idea_market::Column::IdeaId.eq(project.idea_id))
                        .to_owned(),
                ),
            )
            .exec(&txn)
            .await?;

        let mut project = project.into_active_model();
        project.team_id = Set(new_team_id);
        project.update(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    pub async fn delete(state: &AppState, project_id: Uuid) -> Result<(), AppError> {
        let today = Local::now().date_naive();
        let txn = state.conn.begin().await?;

        let project = entity::prelude::Project::find_by_id(project_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut project_active = project.clone().into_active_model();
        project_active.status = Set(ProjectStatus::Deleted);
        project_active.update(&txn).await?;

        entity::prelude::ProjectMember::update_many()
            .col_expr(project_member::Column::FinishDate, Expr::value(Some(today)))
            .filter(project_member::Column::ProjectId.eq(project_id))
            .filter(project_member::Column::FinishDate.is_null())
            .exec(&txn)
            .await?;

        entity::prelude::IdeaMarket::update_many()
            .col_expr(idea_market::Column::TeamId, Expr::value(None::<Uuid>))
            .col_expr(
                idea_market::Column::Status,
                Expr::value(entity::idea_market_status::IdeaMarketStatus::RecruitmentIsOpen),
            )
            .filter(idea_market::Column::IdeaId.eq(project.idea_id))
            .exec(&txn)
            .await?;

        entity::prelude::Team::update_many()
            .col_expr(team::Column::HasActiveProject, Expr::value(false))
            .filter(team::Column::Id.eq(project.team_id))
            .exec(&txn)
            .await?;

        entity::prelude::TeamMarketRequest::update_many()
            .col_expr(
                team_market_request::Column::Status,
                Expr::value(entity::request_status::RequestStatus::Annulled),
            )
            .filter(team_market_request::Column::TeamId.eq(project.team_id))
            .filter(
                team_market_request::Column::IdeaMarketId.in_subquery(
                    Query::select()
                        .column(idea_market::Column::Id)
                        .from(idea_market::Entity)
                        .and_where(idea_market::Column::IdeaId.eq(project.idea_id))
                        .to_owned(),
                ),
            )
            .exec(&txn)
            .await?;

        txn.commit().await?;
        Ok(())
    }
}
