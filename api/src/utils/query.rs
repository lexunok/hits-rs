/// Shared query helpers used across multiple services.
use std::collections::HashMap;

use crate::{AppState, dtos::tag::TagDto, error::AppError};
use entity::{tag, task_tag};
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, JoinType, QueryFilter, QuerySelect, RelationTrait, prelude::Uuid};

/// Shared query-result row for tag loading (used in project, task, sprint, task_movement_log).
#[derive(Debug, FromQueryResult)]
pub struct TagRow {
    pub task_id: Uuid,
    pub tag_id: Option<Uuid>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub tag_confirmed: Option<bool>,
    pub tag_creator_id: Option<Uuid>,
    pub tag_updater_id: Option<Uuid>,
    pub tag_deleter_id: Option<Uuid>,
}

/// Load tags for a set of task IDs, returning a map from task_id → Vec<TagDto>.
pub async fn load_tags_for_tasks(
    state: &AppState,
    task_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<TagDto>>, AppError> {
    if task_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows: Vec<TagRow> = entity::prelude::TaskTag::find()
        .filter(task_tag::Column::TaskId.is_in(task_ids.iter().copied()))
        .join(JoinType::InnerJoin, task_tag::Relation::Tag.def())
        .column_as(task_tag::Column::TaskId, "task_id")
        .column_as(tag::Column::Id, "tag_id")
        .column_as(tag::Column::Name, "tag_name")
        .column_as(tag::Column::Color, "tag_color")
        .column_as(tag::Column::Confirmed, "tag_confirmed")
        .column_as(tag::Column::CreatorId, "tag_creator_id")
        .column_as(tag::Column::UpdaterId, "tag_updater_id")
        .column_as(tag::Column::DeleterId, "tag_deleter_id")
        .into_model()
        .all(&state.conn)
        .await?;

    let mut map = HashMap::<Uuid, Vec<TagDto>>::new();
    for row in rows {
        let Some(tag_id) = row.tag_id else { continue };
        let entry = map.entry(row.task_id).or_default();
        if entry.iter().all(|t| t.id != tag_id) {
            entry.push(TagDto {
                id: tag_id,
                name: row.tag_name.unwrap_or_default(),
                color: row.tag_color.unwrap_or_default(),
                confirmed: row.tag_confirmed.unwrap_or(false),
                creator_id: row.tag_creator_id,
                updater_id: row.tag_updater_id,
                deleter_id: row.tag_deleter_id,
            });
        }
    }
    Ok(map)
}
