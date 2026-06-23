use crate::{
    AppState,
    dtos::{
        common::{PaginatedResponse, PaginationParams},
        company::{CompanyResponse, CreateCompanyRequest, UpdateCompanyRequest},
        user::UserDto,
    },
    error::AppError,
};
use entity::{
    company, company_member,
    prelude::{Company, CompanyMember, Users},
    users,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, ExprTrait, IntoActiveModel, JoinType, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, TransactionTrait, prelude::Uuid
};

pub struct CompanyService;

impl CompanyService {
    pub async fn get_all(
        state: &AppState,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<CompanyResponse>, AppError> {

        let mut condition = Condition::all();

        if let Some(search_text) = pagination.search_text {
            condition = condition.add(company::Column::Name.ilike(format!("%{}%", search_text)));
        }

        let query = Company::find()
            .join(JoinType::InnerJoin, company::Relation::Owner.def())
            .filter(condition)
            .into_partial_model::<CompanyResponse>();

        let paginator = query.paginate(&state.conn, pagination.page_size);
        let count = paginator.num_items().await.unwrap_or(0);
        let list = paginator.fetch_page(pagination.page).await.unwrap_or_default();

        Ok(PaginatedResponse { count, list })
    }
    pub async fn get_members(
        state: &AppState,
        id: Uuid,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<UserDto>, AppError> {

        let mut condition = Condition::all().add(company_member::Column::CompanyId.eq(id));

        if let Some(search_text) = pagination.search_text {
            condition = condition.add(users::Column::FirstName.ilike(format!("%{}%", search_text)).or(users::Column::LastName.ilike(format!("%{}%", search_text))));
        }

        let query = Users::find()
            .inner_join(CompanyMember)
            .filter(condition)
            .into_partial_model();

        let paginator = query.paginate(&state.conn, pagination.page_size);
        let count = paginator.num_items().await.unwrap_or(0);
        let list = paginator.fetch_page(pagination.page).await.unwrap_or_default();

        Ok(PaginatedResponse { count, list })
    }

    // pub async fn get_my(state: &AppState, user_id: Uuid) -> Vec<CompanyResponse> {
    //     Company::find()
    //         .join(JoinType::InnerJoin, company::Relation::Owner.def())
    //         .filter(
    //             company::Column::Id
    //                 .in_subquery(
    //                     sea_query::Query::select()
    //                         .column(company_member::Column::CompanyId)
    //                         .from(company_member::Entity)
    //                         .and_where(company_member::Column::UserId.eq(user_id))
    //                         .to_owned(),
    //                 )
    //                 .or(company::Column::OwnerId.eq(user_id)),
    //         )
    //         .into_partial_model::<CompanyResponse>()
    //         .all(&state.conn)
    //         .await
    //         .unwrap_or_default()
    // }
    pub async fn get_one(state: &AppState, id: Uuid) -> Result<CompanyResponse, AppError> {
        let mut company: CompanyResponse = Company::find_by_id(id)
            .join(JoinType::InnerJoin, company::Relation::Owner.def())
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let members: Vec<UserDto> = Users::find()
            .inner_join(CompanyMember)
            .filter(company_member::Column::CompanyId.eq(id))
            .into_partial_model()
            .all(&state.conn)
            .await?;

        company.members = members;

        Ok(company)
    }
    pub async fn get_one_by_name(state: &AppState, name: String) -> Result<CompanyResponse, AppError> {
        let mut company: CompanyResponse = Company::find()
            .filter(company::Column::Name.eq(name))
            .join(JoinType::InnerJoin, company::Relation::Owner.def())
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let members: Vec<UserDto> = Users::find()
            .inner_join(CompanyMember)
            .filter(company_member::Column::CompanyId.eq(company.id))
            .into_partial_model()
            .all(&state.conn)
            .await?;

        company.members = members;

        Ok(company)
    }
    pub async fn create(
        state: &AppState,
        payload: CreateCompanyRequest,
    ) -> Result<CompanyResponse, AppError> {
        let txn = state.conn.begin().await?;

        let new_company = company::ActiveModel {
            name: Set(payload.name),
            owner_id: Set(payload.owner_id),
            ..Default::default()
        };
        let new_company = new_company.insert(&txn).await?;

        let members: Vec<company_member::ActiveModel> = payload
            .members
            .iter()
            .map(|member| company_member::ActiveModel {
                user_id: Set(member.to_owned()),
                company_id: Set(new_company.id),
            })
            .collect();

        let members_keys: Vec<Uuid> = CompanyMember::insert_many(members)
            .exec_with_returning(&txn)
            .await?
            .iter()
            .map(|m| m.user_id)
            .collect();

        let mut company: CompanyResponse = Company::find_by_id(new_company.id)
            .join(JoinType::InnerJoin, company::Relation::Owner.def())
            .into_partial_model()
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let members: Vec<UserDto> = Users::find()
            .filter(users::Column::Id.is_in(members_keys))
            .into_partial_model()
            .all(&txn)
            .await?;

        company.members = members;

        txn.commit().await?;

        Ok(company)
    }

    pub async fn update(
        state: &AppState,
        payload: UpdateCompanyRequest,
    ) -> Result<CompanyResponse, AppError> {
        let txn = state.conn.begin().await?;

        let mut company = Company::find_by_id(payload.id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?
            .into_active_model();

        if let Some(name) = payload.name {
            if !name.is_empty() {
                company.name = Set(name);
            }
        }

        if let Some(owner_id) = payload.owner_id {
            company.owner_id = Set(owner_id);
        }
        company.update(&txn).await?;

        let mut company: CompanyResponse = Company::find_by_id(payload.id)
            .join(JoinType::InnerJoin, company::Relation::Owner.def())
            .into_partial_model()
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        if let Some(members) = payload.remove_members {
            CompanyMember::delete_many()
                .filter(company_member::Column::CompanyId.eq(payload.id))
                .filter(company_member::Column::UserId.is_in(members))
                .exec(&txn)
                .await?;
        }

        if let Some(members) = payload.new_members {
            let members: Vec<company_member::ActiveModel> = members
                .iter()
                .map(|member| company_member::ActiveModel {
                    user_id: Set(member.to_owned()),
                    company_id: Set(payload.id),
                })
                .collect();

            CompanyMember::insert_many(members)
                .exec(&txn)
                .await?;
        }

        let members: Vec<UserDto> = Users::find()
            .inner_join(CompanyMember)
            .filter(company_member::Column::CompanyId.eq(payload.id))
            .into_partial_model()
            .all(&txn)
            .await?;

        company.members = members;

        txn.commit().await?;

        Ok(company)
    }
    pub async fn delete(state: &AppState, id: Uuid) -> Result<(), AppError> {
        Company::delete_by_id(id).exec(&state.conn).await?;
        Ok(())
    }
}
