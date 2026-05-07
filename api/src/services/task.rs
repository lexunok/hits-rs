use chrono::Local;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, ExprTrait, IntoActiveModel,
    JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait, TransactionTrait,
    prelude::Uuid,
    sea_query::Expr,
};

use crate::{
    AppState,
    dtos::{
        project::TaskDto,
        task::{CreateTaskRequest, UpdateTaskRequest},
        user::UserDto,
    },
    error::AppError,
    utils::query::load_tags_for_tasks,
};
use entity::{task, task_movement_log, task_status::TaskStatus, task_tag, users};

pub struct TaskService;

impl TaskService {
    /// Загружает задачи по произвольному фильтру, с исполнителями и тегами.
    async fn load_tasks(
        state: &AppState,
        filter: sea_orm::Condition,
    ) -> Result<Vec<TaskDto>, AppError> {
        let initiator = sea_orm::sea_query::Alias::new("initiator");
        let executor = sea_orm::sea_query::Alias::new("executor");

        let rows: Vec<TaskRow> = entity::prelude::Task::find()
            .filter(filter)
            .join_as(JoinType::LeftJoin, task::Relation::Users1.def(), initiator.clone())
            .join_as(JoinType::LeftJoin, task::Relation::Users2.def(), executor.clone())
            .tbl_col_as((initiator.clone(), users::Column::Id), "initiator_id")
            .tbl_col_as((initiator.clone(), users::Column::Email), "initiator_email")
            .tbl_col_as((initiator.clone(), users::Column::FirstName), "initiator_first_name")
            .tbl_col_as((initiator.clone(), users::Column::LastName), "initiator_last_name")
            .tbl_col_as((executor.clone(), users::Column::Id), "executor_id")
            .tbl_col_as((executor.clone(), users::Column::Email), "executor_email")
            .tbl_col_as((executor.clone(), users::Column::FirstName), "executor_first_name")
            .tbl_col_as((executor.clone(), users::Column::LastName), "executor_last_name")
            .order_by_asc(task::Column::Position)
            .order_by_asc(task::Column::StartDate)
            .into_model()
            .all(&state.conn)
            .await?;

        let task_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let tags_map = load_tags_for_tasks(state, &task_ids).await?;

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
                initiator: row.initiator_id.map(|id| {
                    UserDto::from_parts(
                        id,
                        row.initiator_email.unwrap_or_default(),
                        row.initiator_first_name.unwrap_or_default(),
                        row.initiator_last_name.unwrap_or_default(),
                    )
                }),
                executor: row.executor_id.map(|id| {
                    UserDto::from_parts(
                        id,
                        row.executor_email.unwrap_or_default(),
                        row.executor_first_name.unwrap_or_default(),
                        row.executor_last_name.unwrap_or_default(),
                    )
                }),
                work_hour: row.work_hour,
                start_date: row.start_date,
                finish_date: row.finish_date,
                tags: tags_map.get(&row.id).cloned().unwrap_or_default(),
                status: row.status,
            })
            .collect();

        Ok(tasks)
    }

    // ─── Публичные методы ────────────────────────────────────────────────────

    pub async fn get_all_by_project(
        state: &AppState,
        project_id: Uuid,
    ) -> Result<Vec<TaskDto>, AppError> {
        Self::load_tasks(state, sea_orm::Condition::all().add(task::Column::ProjectId.eq(project_id))).await
    }

    pub async fn get_backlog(
        state: &AppState,
        project_id: Uuid,
    ) -> Result<Vec<TaskDto>, AppError> {
        Self::load_tasks(
            state,
            sea_orm::Condition::all()
                .add(task::Column::ProjectId.eq(project_id))
                .add(task::Column::Status.eq(TaskStatus::InBackLog)),
        )
        .await
    }

    pub async fn get_by_sprint(
        state: &AppState,
        sprint_id: Uuid,
    ) -> Result<Vec<TaskDto>, AppError> {
        Self::load_tasks(state, sea_orm::Condition::all().add(task::Column::SprintId.eq(sprint_id))).await
    }

    pub async fn get_one(state: &AppState, task_id: Uuid) -> Result<TaskDto, AppError> {
        Self::load_tasks(state, sea_orm::Condition::all().add(task::Column::Id.eq(task_id)))
            .await?
            .into_iter()
            .next()
            .ok_or(AppError::NotFound)
    }

    /// Создать задачу. Если sprint_id не указан — помещается в бэклог (position = последняя + 1).
    /// Пишет первую запись в TaskMovementLog.
    pub async fn create(
        state: &AppState,
        payload: CreateTaskRequest,
        initiator_id: Uuid,
    ) -> Result<TaskDto, AppError> {
        let txn = state.conn.begin().await?;
        let now = Local::now();

        // Позиция только для бэклога
        let position = if payload.sprint_id.is_none() {
            let count = entity::prelude::Task::find()
                .filter(task::Column::ProjectId.eq(payload.project_id))
                .filter(task::Column::Status.eq(TaskStatus::InBackLog))
                .all(&txn)
                .await?
                .len() as i32;
            Some(count + 1)
        } else {
            None
        };

        let status = if payload.sprint_id.is_none() {
            TaskStatus::InBackLog
        } else {
            TaskStatus::NewTask
        };

        let task = task::ActiveModel {
            sprint_id: Set(payload.sprint_id),
            project_id: Set(payload.project_id),
            position: Set(position),
            name: Set(payload.name),
            description: Set(payload.description),
            work_hour: Set(payload.work_hour),
            start_date: Set(payload.start_date),
            finish_date: Set(payload.finish_date),
            initiator_id: Set(Some(initiator_id)),
            executor_id: Set(None),
            status: Set(Some(status.clone())),
            leader_comment: Set(None),
            executor_comment: Set(None),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        // Пишем лог движения
        task_movement_log::ActiveModel {
            task_id: Set(task.id),
            executor_id: Set(None),
            user_id: Set(Some(initiator_id)),
            start_date: Set(now.fixed_offset()),
            end_date: Set(None),
            status: Set(Some(status)),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        // Привязываем теги
        for tag_id in &payload.tags {
            entity::task_tag::ActiveModel {
                task_id: Set(task.id),
                tag_id: Set(*tag_id),
            }
            .insert(&txn)
            .await?;
        }

        txn.commit().await?;

        Self::get_one(state, task.id).await
    }

    /// Обновить задачу: name, description, work_hour, теги.
    pub async fn update(
        state: &AppState,
        task_id: Uuid,
        payload: UpdateTaskRequest,
    ) -> Result<(), AppError> {
        let txn = state.conn.begin().await?;

        let mut t = entity::prelude::Task::find_by_id(task_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        t.name = Set(payload.name);
        t.description = Set(payload.description);
        t.work_hour = Set(payload.work_hour);
        t.update(&txn).await?;

        // Перезаписываем теги
        entity::prelude::TaskTag::delete_many()
            .filter(task_tag::Column::TaskId.eq(task_id))
            .exec(&txn)
            .await?;

        for tag_id in &payload.tags {
            entity::task_tag::ActiveModel {
                task_id: Set(task_id),
                tag_id: Set(*tag_id),
            }
            .insert(&txn)
            .await?;
        }

        txn.commit().await?;
        Ok(())
    }

    pub async fn update_executor(
        state: &AppState,
        task_id: Uuid,
        executor_id: Uuid,
    ) -> Result<(), AppError> {
        let mut t = entity::prelude::Task::find_by_id(task_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        t.executor_id = Set(Some(executor_id));
        t.update(&state.conn).await?;
        Ok(())
    }

    pub async fn update_leader_comment(
        state: &AppState,
        task_id: Uuid,
        comment: String,
    ) -> Result<(), AppError> {
        let mut t = entity::prelude::Task::find_by_id(task_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        t.leader_comment = Set(Some(comment));
        t.update(&state.conn).await?;
        Ok(())
    }

    pub async fn update_executor_comment(
        state: &AppState,
        task_id: Uuid,
        comment: String,
    ) -> Result<(), AppError> {
        let mut t = entity::prelude::Task::find_by_id(task_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        t.executor_comment = Set(Some(comment));
        t.update(&state.conn).await?;
        Ok(())
    }

    /// Переместить задачу в бэклоге (сдвигает позиции соседних).
    pub async fn move_position(
        state: &AppState,
        task_id: Uuid,
        new_position: i32,
    ) -> Result<(), AppError> {
        let txn = state.conn.begin().await?;

        let task_model = entity::prelude::Task::find_by_id(task_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let old_position = task_model.position.unwrap_or(new_position);
        let project_id = task_model.project_id;

        if old_position == new_position {
            return Ok(());
        }

        if old_position > new_position {
            // Сдвигаем вверх: задачи между [new_pos, old_pos) получают +1
            entity::prelude::Task::update_many()
                .col_expr(task::Column::Position, Expr::col(task::Column::Position).add(1))
                .filter(task::Column::ProjectId.eq(project_id))
                .filter(task::Column::Id.ne(task_id))
                .filter(task::Column::Status.eq(TaskStatus::InBackLog))
                .filter(task::Column::Position.gte(new_position))
                .filter(task::Column::Position.lt(old_position))
                .exec(&txn)
                .await?;
        } else {
            // Сдвигаем вниз: задачи между (old_pos, new_pos] получают -1
            entity::prelude::Task::update_many()
                .col_expr(task::Column::Position, Expr::col(task::Column::Position).sub(1))
                .filter(task::Column::ProjectId.eq(project_id))
                .filter(task::Column::Id.ne(task_id))
                .filter(task::Column::Status.eq(TaskStatus::InBackLog))
                .filter(task::Column::Position.gt(old_position))
                .filter(task::Column::Position.lte(new_position))
                .exec(&txn)
                .await?;
        }

        let mut t = task_model.into_active_model();
        t.position = Set(Some(new_position));
        t.update(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    /// Удалить задачу. Если задача в бэклоге — пересчитываем позиции.
    pub async fn delete(state: &AppState, task_id: Uuid) -> Result<(), AppError> {
        let txn = state.conn.begin().await?;

        let task_model = entity::prelude::Task::find_by_id(task_id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let was_in_backlog = task_model.status == Some(TaskStatus::InBackLog);
        let old_position = task_model.position;
        let project_id = task_model.project_id;

        entity::prelude::Task::delete_by_id(task_id)
            .exec(&txn)
            .await?;

        if was_in_backlog {
            if let Some(pos) = old_position {
                entity::prelude::Task::update_many()
                    .col_expr(
                        task::Column::Position,
                        Expr::col(task::Column::Position).sub(1),
                    )
                    .filter(task::Column::ProjectId.eq(project_id))
                    .filter(task::Column::Status.eq(TaskStatus::InBackLog))
                    .filter(task::Column::Position.gt(pos))
                    .exec(&txn)
                    .await?;
            }
        }

        txn.commit().await?;
        Ok(())
    }
}

// ─── Query result structs ────────────────────────────────────────────────────

#[derive(Debug, sea_orm::FromQueryResult)]
struct TaskRow {
    pub id: Uuid,
    pub sprint_id: Option<Uuid>,
    pub project_id: Uuid,
    pub position: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub leader_comment: Option<String>,
    pub executor_comment: Option<String>,
    pub work_hour: Option<f64>,
    pub start_date: sea_orm::prelude::Date,
    pub finish_date: Option<sea_orm::prelude::Date>,
    pub status: Option<TaskStatus>,
    pub initiator_id: Option<Uuid>,
    pub initiator_email: Option<String>,
    pub initiator_first_name: Option<String>,
    pub initiator_last_name: Option<String>,
    pub executor_id: Option<Uuid>,
    pub executor_email: Option<String>,
    pub executor_first_name: Option<String>,
    pub executor_last_name: Option<String>,
}
