use crate::{
    AppState,
    dtos::{
        common::PaginatedResponse,
        user::{UserCreatePayload, UserDto, UserPaginationParams, UserUpdatePayload},
    },
    error::AppError,
    utils::security::hash_password,
};
use entity::{
    prelude::{Skill, TeamMember, Users},
    team_member, users,
};
use itertools::Itertools;
use sea_orm::{
    sea_query::{Expr, ExprTrait, BinOper, extension::postgres::PgBinOper},
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, IntoActiveModel, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, prelude::Uuid,
};
use serde_json::json;

pub struct UserService;

impl UserService {
    pub async fn get_all(
        state: &AppState,
        pagination: UserPaginationParams,
    ) -> Result<PaginatedResponse<UserDto>, AppError> {
        let mut condition = Condition::all().add(users::Column::IsDeleted.eq(false));
        let mut order = Order::Asc;

        if let Some(search_text) = pagination.search_text {
            condition = condition.add(users::Column::FirstName.ilike(format!("%{}%", search_text))
                .or(users::Column::LastName.ilike(format!("%{}%", search_text))));
        }
        if let Some(ignored_ids) = pagination.ignored_ids {
            condition = condition.add(users::Column::Id.is_not_in(ignored_ids));
        }
        if let Some(ignored_team) = pagination.ignored_team {
            condition = condition.add(team_member::Column::TeamId.ne(ignored_team));
        }
        if let Some(selected_roles) = pagination.selected_roles {
            condition = condition.add(
                Expr::col(users::Column::Roles)
                    .binary(BinOper::PgOperator(PgBinOper::Overlap), Expr::val(selected_roles))
            );
        }
        if let Some(by_descending) = pagination.by_descending {
            if by_descending {
                order = Order::Desc
            }
        }

        let query = Users::find()
            .inner_join(TeamMember)
            .filter(condition)
            .order_by(users::Column::CreatedAt, order)
            .into_partial_model();

        let paginator = query.paginate(&state.conn, pagination.page_size);
        let count = paginator.num_items().await.unwrap_or(0);
        let list = paginator.fetch_page(pagination.page).await.unwrap_or_default();

        Ok(PaginatedResponse { count, list })
    }
    pub async fn get_all_with_skills(state: &AppState) -> Vec<UserDto> {
        Users::load()
            .with(Skill)
            .all(&state.conn)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect()
    }
    pub async fn get_all_in_teams(state: &AppState) -> Vec<UserDto> {
        TeamMember::load()
            .filter(TeamMember::COLUMN.finish_date.is_null())
            .with(Users)
            .order_by_desc(Users::COLUMN.created_at)
            .all(&state.conn)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| m.member.into_option())
            .unique_by(|u| u.id)
            .map(|member| UserDto {
                id: member.id,
                first_name: member.first_name,
                last_name: member.last_name,
                email: member.email,
                ..Default::default()
            })
            .collect()
    }
    pub async fn get_all_not_in_teams(state: &AppState) -> Vec<UserDto> {
        Users::find()
            .left_join(TeamMember)
            .filter(Expr::not(Expr::exists(
                TeamMember::find()
                    .select_only()
                    .expr(Expr::val(1))
                    .filter(
                        Expr::col((team_member::Entity, team_member::Column::UserId))
                            .eq(Expr::col((users::Entity, users::Column::Id))),
                    )
                    .filter(
                        Expr::col((team_member::Entity, team_member::Column::FinishDate)).is_null(),
                    )
                    .into_query(),
            )))
            .into_partial_model()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }
    pub async fn get_one(state: &AppState, id: Uuid) -> Result<UserDto, AppError> {
        Users::find_by_id(id)
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)
    }
    pub async fn create(state: &AppState, payload: UserCreatePayload) -> Result<(), AppError> {
        let mut user =
            users::ActiveModel::from_json(json!(payload)).map_err(|_| AppError::BadRequest)?;

        user.email = Set(payload.email.to_lowercase());
        user.password = Set(hash_password(&payload.password)?);

        user.insert(&state.conn).await?;

        Ok(())
    }
    pub async fn update(state: &AppState, payload: UserUpdatePayload) -> Result<(), AppError> {
        let mut user =
            users::ActiveModel::from_json(json!(payload)).map_err(|_| AppError::BadRequest)?;

        user.email = Set(payload.email.to_lowercase());

        user.update(&state.conn).await?;

        Ok(())
    }
    pub async fn restore(state: &AppState, email: String) -> Result<(), AppError> {
        let mut user = Users::find_by_email(email)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        user.is_deleted = Set(false);

        user.update(&state.conn).await?;

        Ok(())
    }
    pub async fn delete(state: &AppState, id: Uuid) -> Result<(), AppError> {
        let mut user = Users::find_by_id(id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        user.is_deleted = Set(true);

        user.update(&state.conn).await?;

        Ok(())
    }
}
