use chrono::Local;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait,
    prelude::Uuid,
};

use crate::{
    AppState,
    dtos::{
        project::TaskDto,
        tag::TagDto,
        task_movement_log::{MoveTaskRequest, TaskMovementLogDto},
        user::UserDto,
    },
    error::AppError,
    utils::query::load_tags_for_tasks,
};
use entity::{prelude::*, task, task_movement_log, task_status::TaskStatus, users};

pub struct TaskMovementLogService;

impl TaskMovementLogService {
    pub async fn get_all_by_task(
        state: &AppState,
        task_id: Uuid,
    ) -> Result<Vec<TaskMovementLogDto>, AppError> {
        let initiator = sea_orm::sea_query::Alias::new("t_initiator");
        let executor_task = sea_orm::sea_query::Alias::new("t_executor");
        let executor_log = sea_orm::sea_query::Alias::new("tml_executor");
        let user_log = sea_orm::sea_query::Alias::new("tml_user");

        let rows: Vec<LogRow> = entity::prelude::TaskMovementLog::find()
            .filter(task_movement_log::Column::TaskId.eq(task_id))
            // JOIN task
            .join(JoinType::LeftJoin, task_movement_log::Relation::Task.def())
            // initiator задачи
            .join_as(JoinType::LeftJoin, task::Relation::Users1.def(), initiator.clone())
            // executor задачи
            .join_as(JoinType::LeftJoin, task::Relation::Users2.def(), executor_task.clone())
            // executor лога
            .join_as(JoinType::LeftJoin, task_movement_log::Relation::Users2.def(), executor_log.clone())
            // user лога
            .join_as(JoinType::LeftJoin, task_movement_log::Relation::Users1.def(), user_log.clone())
            // поля task
            .column_as(task::Column::Id, "task_id")
            .column_as(task::Column::SprintId, "task_sprint_id")
            .column_as(task::Column::ProjectId, "task_project_id")
            .column_as(task::Column::Position, "task_position")
            .column_as(task::Column::Name, "task_name")
            .column_as(task::Column::Description, "task_description")
            .column_as(task::Column::LeaderComment, "task_leader_comment")
            .column_as(task::Column::ExecutorComment, "task_executor_comment")
            .column_as(task::Column::WorkHour, "task_work_hour")
            .column_as(task::Column::StartDate, "task_start_date")
            .column_as(task::Column::FinishDate, "task_finish_date")
            .column_as(task::Column::Status, "task_status")
            // initiator задачи
            .tbl_col_as((initiator.clone(), users::Column::Id), "ti_id")
            .tbl_col_as((initiator.clone(), users::Column::Email), "ti_email")
            .tbl_col_as((initiator.clone(), users::Column::FirstName), "ti_first_name")
            .tbl_col_as((initiator.clone(), users::Column::LastName), "ti_last_name")
            // executor задачи
            .tbl_col_as((executor_task.clone(), users::Column::Id), "te_id")
            .tbl_col_as((executor_task.clone(), users::Column::Email), "te_email")
            .tbl_col_as((executor_task.clone(), users::Column::FirstName), "te_first_name")
            .tbl_col_as((executor_task.clone(), users::Column::LastName), "te_last_name")
            // executor лога
            .tbl_col_as((executor_log.clone(), users::Column::Id), "tml_e_id")
            .tbl_col_as((executor_log.clone(), users::Column::Email), "tml_e_email")
            .tbl_col_as((executor_log.clone(), users::Column::FirstName), "tml_e_first_name")
            .tbl_col_as((executor_log.clone(), users::Column::LastName), "tml_e_last_name")
            // user лога
            .tbl_col_as((user_log.clone(), users::Column::Id), "tml_u_id")
            .tbl_col_as((user_log.clone(), users::Column::Email), "tml_u_email")
            .tbl_col_as((user_log.clone(), users::Column::FirstName), "tml_u_first_name")
            .tbl_col_as((user_log.clone(), users::Column::LastName), "tml_u_last_name")
            .order_by_asc(task_movement_log::Column::StartDate)
            .into_model()
            .all(&state.conn)
            .await?;

        let tags: Vec<TagDto> = load_tags_for_tasks(state, &[task_id])
            .await?
            .remove(&task_id)
            .unwrap_or_default();

        let mut result = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for row in rows {
            if !seen_ids.insert(row.id) {
                continue;
            }

            let wasted_time = row.end_date.map(|end| {
                let duration = end.signed_duration_since(row.start_date);
                let total_minutes = duration.num_minutes();
                let hours = total_minutes / 60;
                let minutes = total_minutes % 60;
                format!(
                    "{:02}ч {:02}мин",
                    hours, minutes
                )
            });

            let task_dto = TaskDto {
                id: row.task_id,
                sprint_id: row.task_sprint_id,
                project_id: row.task_project_id,
                position: row.task_position,
                name: row.task_name.clone(),
                description: row.task_description.clone(),
                leader_comment: row.task_leader_comment.clone(),
                executor_comment: row.task_executor_comment.clone(),
                initiator: UserDto::from_parts_opt(row.ti_id, row.ti_email.clone(), row.ti_first_name.clone(), row.ti_last_name.clone()),
                executor: UserDto::from_parts_opt(row.te_id, row.te_email.clone(), row.te_first_name.clone(), row.te_last_name.clone()),
                work_hour: row.task_work_hour,
                start_date: row.task_start_date,
                finish_date: row.task_finish_date,
                tags: tags.clone(),
                status: row.task_status,
            };

            result.push(TaskMovementLogDto {
                id: row.id,
                task: task_dto,
                executor: UserDto::from_parts_opt(row.tml_e_id, row.tml_e_email, row.tml_e_first_name, row.tml_e_last_name),
                user: UserDto::from_parts_opt(row.tml_u_id, row.tml_u_email, row.tml_u_first_name, row.tml_u_last_name),
                start_date: row.start_date,
                end_date: row.end_date,
                wasted_time,
                status: row.status,
            });
        }

        Ok(result)
    }

    pub async fn move_task(
        state: &AppState,
        payload: MoveTaskRequest,
    ) -> Result<TaskMovementLogDto, AppError> {
        let now = Local::now().fixed_offset();

        TaskMovementLog::update_many()
            .col_expr(
                task_movement_log::Column::EndDate,
                sea_orm::sea_query::Expr::value(Some(now)),
            )
            .filter(task_movement_log::Column::TaskId.eq(payload.task_id))
            .filter(task_movement_log::Column::EndDate.is_null())
            .exec(&state.conn)
            .await?;

        let mut task = Task::find_by_id(payload.task_id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        if payload.status == TaskStatus::NewTask {
            task.executor_id = Set(None);
        }
        task.status = Set(Some(payload.status.clone()));
        task.update(&state.conn).await?;

        let executor_id = if payload.status == TaskStatus::NewTask {
            None
        } else {
            payload.executor_id
        };

        let log = task_movement_log::ActiveModel {
            task_id: Set(payload.task_id),
            executor_id: Set(executor_id),
            user_id: Set(payload.user_id),
            start_date: Set(now),
            end_date: Set(None),
            status: Set(Some(payload.status.clone())),
            ..Default::default()
        }
        .insert(&state.conn)
        .await?;

        let task_dto = Self::get_task_dto(state, payload.task_id).await?;

        Ok(TaskMovementLogDto {
            id: log.id,
            task: task_dto,
            executor: None,
            user: None,
            start_date: log.start_date,
            end_date: log.end_date,
            wasted_time: None,
            status: log.status,
        })
    }

    async fn get_task_dto(state: &AppState, task_id: Uuid) -> Result<TaskDto, AppError> {
        crate::services::task::TaskService::get_one(state, task_id).await
    }
}

// ─── Query result structs ─────────────────────────────────────────────────────

#[derive(Debug, sea_orm::FromQueryResult)]
struct LogRow {
    pub id: Uuid,
    pub start_date: sea_orm::prelude::DateTimeWithTimeZone,
    pub end_date: Option<sea_orm::prelude::DateTimeWithTimeZone>,
    pub status: Option<TaskStatus>,
    // task fields
    pub task_id: Uuid,
    pub task_sprint_id: Option<Uuid>,
    pub task_project_id: Uuid,
    pub task_position: Option<i32>,
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_leader_comment: Option<String>,
    pub task_executor_comment: Option<String>,
    pub task_work_hour: Option<f64>,
    pub task_start_date: sea_orm::prelude::Date,
    pub task_finish_date: Option<sea_orm::prelude::Date>,
    pub task_status: Option<TaskStatus>,
    // task initiator
    pub ti_id: Option<Uuid>,
    pub ti_email: Option<String>,
    pub ti_first_name: Option<String>,
    pub ti_last_name: Option<String>,
    // task executor
    pub te_id: Option<Uuid>,
    pub te_email: Option<String>,
    pub te_first_name: Option<String>,
    pub te_last_name: Option<String>,
    // log executor
    pub tml_e_id: Option<Uuid>,
    pub tml_e_email: Option<String>,
    pub tml_e_first_name: Option<String>,
    pub tml_e_last_name: Option<String>,
    // log user
    pub tml_u_id: Option<Uuid>,
    pub tml_u_email: Option<String>,
    pub tml_u_first_name: Option<String>,
    pub tml_u_last_name: Option<String>,
}


