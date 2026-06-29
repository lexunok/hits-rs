use crate::{
    AppState,
    dtos::{
        common::PaginatedResponse,
        group::{CreateGroupRequest, GroupDto, GroupPaginationParams, UpdateGroupRequest},
        user::UserDto,
    },
    error::AppError,
};
use entity::{
    group, group_member,
    prelude::{Group, GroupMember, Users},
    users,
};
use sea_orm::{
    sea_query::{Expr, ExprTrait, BinOper, extension::postgres::PgBinOper},
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, TransactionTrait, prelude::Uuid, sea_query
};

pub struct GroupService;

impl GroupService {
    pub async fn get_all(
        state: &AppState,
        pagination: GroupPaginationParams,
    ) -> Result<PaginatedResponse<GroupDto>, AppError> {

        let mut condition = Condition::all();

        if let Some(search_text) = pagination.search_text {
            condition = condition.add(group::Column::Name.ilike(format!("%{}%", search_text)));
        }
        if let Some(selected_roles) = pagination.selected_roles {
            condition = condition.add(
                Expr::col(group::Column::Roles)
                    .binary(BinOper::PgOperator(PgBinOper::Overlap), Expr::val(selected_roles))
            );
        }

        let query = Group::find().filter(condition).into_partial_model();

        let paginator = query.paginate(&state.conn, pagination.page_size);
        let count = paginator.num_items().await.unwrap_or(0);
        let list = paginator.fetch_page(pagination.page).await.unwrap_or_default();

        Ok(PaginatedResponse { count, list })
    }

    pub async fn get_one(state: &AppState, id: Uuid) -> Result<GroupDto, AppError> {
        let mut group: GroupDto = Group::find_by_id(id)
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let members: Vec<UserDto> = Users::find()
            .filter(
                users::Column::Id.in_subquery(
                    sea_query::Query::select()
                        .column(group_member::Column::UserId)
                        .from(group_member::Entity)
                        .and_where(group_member::Column::GroupId.eq(id))
                        .to_owned(),
                ),
            )
            .into_partial_model()
            .all(&state.conn)
            .await?;

        group.members = members;

        Ok(group)
    }

    pub async fn create(
        state: &AppState,
        payload: CreateGroupRequest,
    ) -> Result<GroupDto, AppError> {
        let txn = state.conn.begin().await?;

        let new_group = group::ActiveModel {
            name: Set(payload.name),
            roles: Set(payload.roles),
            ..Default::default()
        };
        let new_group = new_group.insert(&txn).await?;

        let members: Vec<group_member::ActiveModel> = payload
            .members
            .iter()
            .map(|member| group_member::ActiveModel {
                user_id: Set(member.to_owned()),
                group_id: Set(new_group.id),
            })
            .collect();

        let members_keys: Vec<Uuid> = GroupMember::insert_many(members)
            .exec_with_returning(&txn)
            .await?
            .iter()
            .map(|m| m.user_id)
            .collect();

        let members: Vec<UserDto> = Users::find()
            .filter(users::Column::Id.is_in(members_keys))
            .into_partial_model()
            .all(&txn)
            .await?;

        txn.commit().await?;

        Ok(GroupDto {
            id: new_group.id,
            name: new_group.name,
            roles: new_group.roles,
            members,
        })
    }

    pub async fn update(
        state: &AppState,
        payload: UpdateGroupRequest,
    ) -> Result<GroupDto, AppError> {
        let txn = state.conn.begin().await?;

        let mut group = Group::find_by_id(payload.id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        if let Some(name) = payload.name {
            if !name.is_empty() {
                group.name = Set(name);
            }
        }

        if let Some(roles) = payload.roles {
            group.roles = Set(roles);
        }

        let group = group.update(&txn).await?;

        if let Some(members) = payload.remove_members {
            GroupMember::delete_many()
                .filter(group_member::Column::UserId.is_in(members))
                .exec(&txn)
                .await?;
        }

        if let Some(members) = payload.new_members {
            let members: Vec<group_member::ActiveModel> = members
                .iter()
                .map(|member| group_member::ActiveModel {
                    user_id: Set(member.to_owned()),
                    group_id: Set(payload.id),
                })
                .collect();

            GroupMember::insert_many(members).exec(&txn).await?;
        }

        txn.commit().await?;

        Ok(Self::get_one(state, group.id).await?)
    }

    pub async fn delete(state: &AppState, id: Uuid) -> Result<(), AppError> {
        Group::delete_by_id(id).exec(&state.conn).await?;
        Ok(())
    }
}
