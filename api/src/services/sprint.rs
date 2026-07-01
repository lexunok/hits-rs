use chrono::{Local, NaiveDate};
use sea_orm::{PaginatorTrait, 
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set, TransactionTrait, prelude::Uuid, sea_query::Expr
};

use crate::{
    AppState,
    dtos::{
        common::{PaginatedResponse, PaginationParams}, project::TaskDto, sprint::{
            AddSprintMarkRequest, CreateSprintRequest, SprintDto, SprintMarkDto,
            UpdateSprintRequest,
        }, user::UserDto
    },
    error::AppError,
    utils::query::load_tags_for_tasks,
};
use entity::{ prelude::Sprint, 
    project_marks, project_role::ProjectRole, sprint, sprint_mark, sprint_status::SprintStatus,
    task, task_history, task_movement_log, task_status::TaskStatus, users,
};

pub struct SprintService;

impl SprintService {
    /// Загружает задачи спринта с исполнителями (из task_history) и тегами.
    async fn get_tasks_for_sprint(
        state: &AppState,
        sprint_id: Uuid,
    ) -> Result<Vec<TaskDto>, AppError> {
        let initiator_alias = sea_orm::sea_query::Alias::new("initiator");
        let executor_alias = sea_orm::sea_query::Alias::new("executor");

        // Задачи с исполнителем из task_history (как в старом коде — текущее состояние)
        let rows: Vec<SprintTaskRow> = entity::prelude::Task::find()
            .filter(task::Column::SprintId.eq(sprint_id))
            .join(JoinType::LeftJoin, task::Relation::TaskHistory.def())
            .join_as(
                JoinType::LeftJoin,
                task::Relation::Users1.def(),
                initiator_alias.clone(),
            )
            .join_as(
                JoinType::LeftJoin,
                task_history::Relation::Users.def(),
                executor_alias.clone(),
            )
            .tbl_col_as((initiator_alias.clone(), users::Column::Id), "initiator_id")
            .tbl_col_as(
                (initiator_alias.clone(), users::Column::Email),
                "initiator_email",
            )
            .tbl_col_as(
                (initiator_alias.clone(), users::Column::FirstName),
                "initiator_first_name",
            )
            .tbl_col_as(
                (initiator_alias.clone(), users::Column::LastName),
                "initiator_last_name",
            )
            .column_as(task_history::Column::ExecutorId, "executor_id")
            .tbl_col_as(
                (executor_alias.clone(), users::Column::Email),
                "executor_email",
            )
            .tbl_col_as(
                (executor_alias.clone(), users::Column::FirstName),
                "executor_first_name",
            )
            .tbl_col_as(
                (executor_alias.clone(), users::Column::LastName),
                "executor_last_name",
            )
            .column_as(task_history::Column::Status, "history_status")
            .column_as(task_history::Column::SprintId, "history_sprint_id")
            .order_by_asc(task::Column::Position)
            .into_model()
            .all(&state.conn)
            .await?;

        let task_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let mut tags_map = load_tags_for_tasks(state, &task_ids).await?;

        let tasks = rows
            .into_iter()
            .map(|row| TaskDto {
                id: row.id,
                sprint_id: row.sprint_id,
                project_id: row.project_id,
                position: row.position,
                name: row.name,
                description: row.description,
                leader_comment: row.leader_comment,
                executor_comment: row.executor_comment,
                initiator: UserDto::from_parts_opt(
                    row.initiator_id,
                    row.initiator_email,
                    row.initiator_first_name,
                    row.initiator_last_name,
                ),
                executor: UserDto::from_parts_opt(
                    row.executor_id,
                    row.executor_email,
                    row.executor_first_name,
                    row.executor_last_name,
                ),
                work_hour: row.work_hour,
                start_date: row.start_date,
                finish_date: row.finish_date,
                tags: tags_map.remove(&row.id).unwrap_or_default(),
                status: row.status,
            })
            .collect();

        Ok(tasks)
    }

    // ─── Публичные методы ────────────────────────────────────────────────────

    pub async fn get_sprints_by_project(
        state: &AppState,
        project_id: Uuid,
        pagination: PaginationParams
    ) -> Result<PaginatedResponse<SprintDto>, AppError> {
        let mut condition = Condition::all();

        if let Some(search_text) = pagination.search_text {
            condition = condition.add(sprint::Column::Name.ilike(format!("%{}%", search_text)));
        }

        let query = Sprint::find()
            .filter(sprint::Column::ProjectId.eq(project_id))
            .filter(condition)
            .order_by_desc(sprint::Column::StartDate);

        let paginator = query.paginate(&state.conn, pagination.page_size);
        let count = paginator.num_items().await.unwrap_or(0);
        let list = paginator
            .fetch_page(pagination.page)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|s| SprintDto {
                id: s.id,
                project_id: s.project_id,
                name: s.name,
                goal: s.goal,
                report: s.report,
                start_date: s.start_date,
                finish_date: s.finish_date,
                working_hours: s.working_hours,
                status: s.status,
                tasks: Vec::new(),
            })
            .collect();
        Ok(PaginatedResponse { count, list })
    }

    pub async fn get_sprint_by_id(
        state: &AppState,
        sprint_id: Uuid,
    ) -> Result<SprintDto, AppError> {
        let sprint = entity::prelude::Sprint::find_by_id(sprint_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let tasks = Self::get_tasks_for_sprint(state, sprint_id).await?;

        Ok(SprintDto {
            id: sprint.id,
            project_id: sprint.project_id,
            name: sprint.name,
            goal: sprint.goal,
            report: sprint.report,
            start_date: sprint.start_date,
            finish_date: sprint.finish_date,
            working_hours: sprint.working_hours,
            status: sprint.status,
            tasks,
        })
    }

    pub async fn get_active_sprint(
        state: &AppState,
        project_id: Uuid,
    ) -> Result<Option<SprintDto>, AppError> {
        let sprint = entity::prelude::Sprint::find()
            .filter(sprint::Column::ProjectId.eq(project_id))
            .filter(sprint::Column::Status.eq(SprintStatus::Active))
            .one(&state.conn)
            .await?;

        let Some(sprint) = sprint else {
            return Ok(None);
        };

        let tasks = Self::get_tasks_for_sprint(state, sprint.id).await?;

        Ok(Some(SprintDto {
            id: sprint.id,
            project_id: sprint.project_id,
            name: sprint.name,
            goal: sprint.goal,
            report: sprint.report,
            start_date: sprint.start_date,
            finish_date: sprint.finish_date,
            working_hours: sprint.working_hours,
            status: sprint.status,
            tasks,
        }))
    }

    pub async fn get_sprint_marks(
        state: &AppState,
        sprint_id: Uuid,
    ) -> Result<Vec<SprintMarkDto>, AppError> {
        let rows: Vec<SprintMarkRow> = entity::prelude::SprintMark::find()
            .filter(sprint_mark::Column::SprintId.eq(sprint_id))
            .join(JoinType::InnerJoin, sprint_mark::Relation::Users.def())
            .column_as(users::Column::FirstName, "first_name")
            .column_as(users::Column::LastName, "last_name")
            .into_model()
            .all(&state.conn)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| SprintMarkDto {
                id: r.id,
                project_id: r.project_id,
                sprint_id: r.sprint_id,
                user_id: r.user_id,
                first_name: r.first_name,
                last_name: r.last_name,
                project_role: r.project_role,
                mark: r.mark,
                count_completed_tasks: r.count_completed_tasks,
            })
            .collect())
    }

    /// Создать спринт: автоматически закрывает текущий активный спринт проекта,
    /// переносит задачи из payload.tasks в новый спринт (статус → NewTask),
    /// логирует движение в TaskMovementLog.
    pub async fn create(
        state: &AppState,
        payload: CreateSprintRequest,
        user_id: Uuid,
    ) -> Result<SprintDto, AppError> {
        let txn = state.conn.begin().await?;
        let now = Local::now();

        // Закрываем активный спринт если есть
        entity::prelude::Sprint::update_many()
            .col_expr(sprint::Column::Status, Expr::value(SprintStatus::Done))
            .filter(sprint::Column::ProjectId.eq(payload.project_id))
            .filter(sprint::Column::Status.eq(SprintStatus::Active))
            .exec(&txn)
            .await?;

        let sprint = sprint::ActiveModel {
            project_id: Set(payload.project_id),
            name: Set(payload.name),
            goal: Set(payload.goal),
            working_hours: Set(payload.working_hours),
            start_date: Set(payload.start_date),
            finish_date: Set(payload.finish_date),
            status: Set(SprintStatus::Active),
            report: Set(None),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        // Переносим задачи из бэклога в спринт
        for task_id in &payload.tasks {
            let task_model = entity::prelude::Task::find_by_id(*task_id)
                .one(&txn)
                .await?;
            let Some(task_model) = task_model else {
                continue;
            };

            let mut task_active = task_model.into_active_model();
            task_active.sprint_id = Set(Some(sprint.id));
            task_active.position = Set(None);
            task_active.status = Set(Some(TaskStatus::NewTask));
            task_active.update(&txn).await?;

            // Закрываем предыдущий открытый лог движения
            entity::prelude::TaskMovementLog::update_many()
                .col_expr(
                    task_movement_log::Column::EndDate,
                    Expr::value(Some(now.fixed_offset())),
                )
                .filter(task_movement_log::Column::TaskId.eq(*task_id))
                .filter(task_movement_log::Column::EndDate.is_null())
                .exec(&txn)
                .await?;

            task_movement_log::ActiveModel {
                task_id: Set(*task_id),
                executor_id: Set(None),
                user_id: Set(Some(user_id)),
                start_date: Set(now.fixed_offset()),
                end_date: Set(None),
                status: Set(Some(TaskStatus::NewTask)),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        // Пересчитываем позиции оставшихся задач бэклога
        let backlog_tasks = entity::prelude::Task::find()
            .filter(task::Column::ProjectId.eq(payload.project_id))
            .filter(task::Column::Status.eq(TaskStatus::InBackLog))
            .order_by_asc(task::Column::Position)
            .all(&txn)
            .await?;

        for (i, bt) in backlog_tasks.into_iter().enumerate() {
            let pos = (i + 1) as i32;
            if bt.position != Some(pos) {
                let mut bt_active = bt.into_active_model();
                bt_active.position = Set(Some(pos));
                bt_active.update(&txn).await?;
            }
        }

        txn.commit().await?;

        Self::get_sprint_by_id(state, sprint.id).await
    }

    /// Обновить спринт: перезаписывает список задач спринта, логирует движение.
    pub async fn update(
        state: &AppState,
        sprint_id: Uuid,
        payload: UpdateSprintRequest,
        user_id: Uuid,
    ) -> Result<SprintDto, AppError> {
        let txn = state.conn.begin().await?;
        let now = Local::now();

        let sprint = entity::prelude::Sprint::find_by_id(sprint_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        // Обновляем поля спринта
        let mut sprint_active = sprint.clone().into_active_model();
        sprint_active.name = Set(payload.name);
        sprint_active.start_date = Set(payload.start_date);
        sprint_active.finish_date = Set(payload.finish_date);
        sprint_active.working_hours = Set(payload.working_hours);
        if let Some(goal) = payload.goal {
            sprint_active.goal = Set(goal);
        }
        sprint_active.update(&txn).await?;

        // Отвязываем все задачи от спринта (вернуть в бэклог)
        let old_sprint_tasks = entity::prelude::Task::find()
            .filter(task::Column::SprintId.eq(sprint_id))
            .all(&txn)
            .await?;

        // Считаем текущий последний position в бэклоге
        let backlog_count = entity::prelude::Task::find()
            .filter(task::Column::ProjectId.eq(sprint.project_id))
            .filter(task::Column::Status.eq(TaskStatus::InBackLog))
            .all(&txn)
            .await?
            .len() as i32;

        let mut next_pos = backlog_count + 1;

        for old_task in old_sprint_tasks {
            if payload.tasks.contains(&old_task.id) {
                // Задача остаётся в спринте — пропускаем возврат
                continue;
            }
            // Возвращаем в бэклог
            let mut t = old_task.clone().into_active_model();
            t.sprint_id = Set(None);
            t.position = Set(Some(next_pos));
            t.status = Set(Some(TaskStatus::InBackLog));
            t.update(&txn).await?;
            next_pos += 1;

            entity::prelude::TaskMovementLog::update_many()
                .col_expr(
                    task_movement_log::Column::EndDate,
                    Expr::value(Some(now.fixed_offset())),
                )
                .filter(task_movement_log::Column::TaskId.eq(old_task.id))
                .filter(task_movement_log::Column::EndDate.is_null())
                .exec(&txn)
                .await?;

            task_movement_log::ActiveModel {
                task_id: Set(old_task.id),
                user_id: Set(Some(user_id)),
                start_date: Set(now.fixed_offset()),
                end_date: Set(None),
                status: Set(Some(TaskStatus::InBackLog)),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        // Новые задачи из payload которых ещё не было в спринте
        let existing_sprint_task_ids: Vec<Uuid> = entity::prelude::Task::find()
            .filter(task::Column::SprintId.eq(sprint_id))
            .all(&txn)
            .await?
            .into_iter()
            .map(|t| t.id)
            .collect();

        for task_id in &payload.tasks {
            if existing_sprint_task_ids.contains(task_id) {
                continue;
            }
            let task_model = entity::prelude::Task::find_by_id(*task_id)
                .one(&txn)
                .await?;
            let Some(task_model) = task_model else {
                continue;
            };
            let was_in_backlog = task_model.status == Some(TaskStatus::InBackLog);
            let new_status = if was_in_backlog {
                TaskStatus::NewTask
            } else {
                task_model.status.clone().unwrap_or(TaskStatus::NewTask)
            };
            let mut t = task_model.into_active_model();
            t.sprint_id = Set(Some(sprint_id));
            t.position = Set(None);
            t.status = Set(Some(new_status.clone()));
            t.update(&txn).await?;

            if was_in_backlog {
                entity::prelude::TaskMovementLog::update_many()
                    .col_expr(
                        task_movement_log::Column::EndDate,
                        Expr::value(Some(now.fixed_offset())),
                    )
                    .filter(task_movement_log::Column::TaskId.eq(*task_id))
                    .filter(task_movement_log::Column::EndDate.is_null())
                    .exec(&txn)
                    .await?;

                task_movement_log::ActiveModel {
                    task_id: Set(*task_id),
                    user_id: Set(Some(user_id)),
                    start_date: Set(now.fixed_offset()),
                    end_date: Set(None),
                    status: Set(Some(new_status)),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;
            }
        }

        txn.commit().await?;

        Self::get_sprint_by_id(state, sprint_id).await
    }

    /// Добавить оценки за спринт (sprint_marks + пересчёт project_marks).
    pub async fn add_marks(
        state: &AppState,
        sprint_id: Uuid,
        project_id: Uuid,
        marks: Vec<AddSprintMarkRequest>,
    ) -> Result<(), AppError> {
        let txn = state.conn.begin().await?;

        for req in marks {
            let count = req.tasks.len() as i32;

            sprint_mark::ActiveModel {
                project_id: Set(project_id),
                sprint_id: Set(sprint_id),
                user_id: Set(req.user_id),
                project_role: Set(req.project_role.clone()),
                mark: Set(req.mark),
                count_completed_tasks: Set(Some(count)),
                ..Default::default()
            }
            .insert(&txn)
            .await?;

            // Пересчитываем project_marks только для не-инициаторов
            if req.project_role != ProjectRole::Initiator {
                let all_marks = entity::prelude::SprintMark::find()
                    .filter(sprint_mark::Column::ProjectId.eq(project_id))
                    .filter(sprint_mark::Column::UserId.eq(req.user_id))
                    .all(&txn)
                    .await?;

                let cnt = all_marks.len() as f64;
                let sum: f64 = all_marks.iter().filter_map(|m| m.mark).sum();
                let avg = if cnt > 0.0 {
                    (sum / cnt * 100.0).floor() / 100.0
                } else {
                    0.0
                };

                let existing = entity::prelude::ProjectMarks::find_by_id((project_id, req.user_id))
                    .one(&txn)
                    .await?;

                if let Some(existing) = existing {
                    let mut pm = existing.into_active_model();
                    pm.mark = Set(Some(avg));
                    pm.update(&txn).await?;
                } else {
                    project_marks::ActiveModel {
                        project_id: Set(project_id),
                        user_id: Set(req.user_id),
                        mark: Set(Some(avg)),
                    }
                    .insert(&txn)
                    .await?;
                }
            }
        }

        txn.commit().await?;
        Ok(())
    }

    /// Завершить спринт: незавершённые задачи → бэклог, report + status=Done, логи.
    pub async fn finish(
        state: &AppState,
        sprint_id: Uuid,
        report: String,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let txn = state.conn.begin().await?;
        let now = Local::now();

        let sprint = entity::prelude::Sprint::find_by_id(sprint_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let tasks = entity::prelude::Task::find()
            .filter(task::Column::SprintId.eq(sprint_id))
            .all(&txn)
            .await?;

        // Считаем сколько задач уже в бэклоге проекта
        let backlog_count = entity::prelude::Task::find()
            .filter(task::Column::ProjectId.eq(sprint.project_id))
            .filter(task::Column::Status.eq(TaskStatus::InBackLog))
            .all(&txn)
            .await?
            .len() as i32;

        let mut next_pos = backlog_count + 1;

        for task_model in tasks {
            // Сохраняем снимок в task_history
            task_history::ActiveModel {
                task_id: Set(task_model.id),
                sprint_id: Set(Some(sprint_id)),
                status: Set(task_model.status.clone()),
                executor_id: Set(task_model.executor_id),
                ..Default::default()
            }
            .insert(&txn)
            .await?;

            if task_model.status != Some(TaskStatus::Done) {
                let mut t = task_model.clone().into_active_model();
                t.sprint_id = Set(None);
                t.position = Set(Some(next_pos));
                t.executor_id = Set(None);
                t.status = Set(Some(TaskStatus::InBackLog));
                t.update(&txn).await?;
                next_pos += 1;

                task_movement_log::ActiveModel {
                    task_id: Set(task_model.id),
                    user_id: Set(Some(user_id)),
                    start_date: Set(now.fixed_offset()),
                    end_date: Set(None),
                    status: Set(Some(TaskStatus::InBackLog)),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;
            }
        }

        let mut sprint_active = sprint.into_active_model();
        sprint_active.status = Set(SprintStatus::Done);
        sprint_active.report = Set(Some(report));
        sprint_active.finish_date = Set(Some(NaiveDate::from(Local::now().date_naive())));
        sprint_active.update(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    /// Удалить спринт: все задачи → бэклог, логи движения.
    pub async fn delete(
        state: &AppState,
        sprint_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let txn = state.conn.begin().await?;
        let now = Local::now();

        let sprint = entity::prelude::Sprint::find_by_id(sprint_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let tasks = entity::prelude::Task::find()
            .filter(task::Column::SprintId.eq(sprint_id))
            .all(&txn)
            .await?;

        let backlog_count = entity::prelude::Task::find()
            .filter(task::Column::ProjectId.eq(sprint.project_id))
            .filter(task::Column::Status.eq(TaskStatus::InBackLog))
            .all(&txn)
            .await?
            .len() as i32;

        let mut next_pos = backlog_count + 1;

        for task_model in tasks {
            let mut t = task_model.clone().into_active_model();
            t.sprint_id = Set(None);
            t.position = Set(Some(next_pos));
            t.executor_id = Set(None);
            t.status = Set(Some(TaskStatus::InBackLog));
            t.update(&txn).await?;
            next_pos += 1;

            task_movement_log::ActiveModel {
                task_id: Set(task_model.id),
                user_id: Set(Some(user_id)),
                start_date: Set(now.fixed_offset()),
                end_date: Set(None),
                status: Set(Some(TaskStatus::InBackLog)),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        entity::prelude::Sprint::delete_by_id(sprint_id)
            .exec(&txn)
            .await?;

        txn.commit().await?;
        Ok(())
    }
}

// ─── Query result structs ────────────────────────────────────────────────────

#[derive(Debug, sea_orm::FromQueryResult)]
struct SprintTaskRow {
    pub id: Uuid,
    pub sprint_id: Option<Uuid>,
    pub project_id: Uuid,
    pub position: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub leader_comment: Option<String>,
    pub executor_comment: Option<String>,
    pub initiator_id: Option<Uuid>,
    pub initiator_email: Option<String>,
    pub initiator_first_name: Option<String>,
    pub initiator_last_name: Option<String>,
    pub executor_id: Option<Uuid>,
    pub executor_email: Option<String>,
    pub executor_first_name: Option<String>,
    pub executor_last_name: Option<String>,
    pub work_hour: Option<f64>,
    pub start_date: sea_orm::prelude::Date,
    pub finish_date: Option<sea_orm::prelude::Date>,
    pub status: Option<TaskStatus>,
    // из task_history — не используются напрямую но нужны для join
    #[allow(dead_code)]
    pub history_status: Option<TaskStatus>,
    #[allow(dead_code)]
    pub history_sprint_id: Option<Uuid>,
}


#[derive(Debug, sea_orm::FromQueryResult)]
struct SprintMarkRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub sprint_id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub project_role: ProjectRole,
    pub mark: Option<f64>,
    pub count_completed_tasks: Option<i32>,
}
