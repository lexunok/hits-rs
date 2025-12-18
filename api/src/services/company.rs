use std::collections::{HashMap, HashSet};

use crate::{
    dtos::{
        company::{CompanyDetailsResponse, CreateCompanyRequest, UpdateCompanyRequest},
        profile::UserDto,
    },
    error::AppError,
    AppState,
};
use entity::{
    company,
    company_member,
    prelude::{Company, CompanyMember, User},
    users,
};
use sea_orm::{
    prelude::Uuid,
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait,
    EntityTrait,
    IntoActiveModel,
    ModelTrait,
    QueryFilter,
    TransactionTrait, QuerySelect, Query,
};
use validator::Validate;

pub struct CompanyService;

impl CompanyService {
    pub async fn create_company(
        state: &AppState,
        payload: CreateCompanyRequest,
    ) -> Result<CompanyDetailsResponse, AppError> {
        payload.validate()?;

        let txn = state.conn.begin().await?;

        // 1. Collect all unique user IDs to verify
        let mut user_ids_to_check: HashSet<Uuid> =
            payload.members.iter().cloned().collect();
        user_ids_to_check.insert(payload.owner_id);

        // 2. Find all existing users from the collected IDs
        let found_users = User::find()
            .filter(users::Column::Id.is_in(user_ids_to_check.clone()))
            .into_model::<UserDto>()
            .all(&txn)
            .await?;

        // 3. Verify that all requested users were found
        if found_users.len() != user_ids_to_check.len() {
            return Err(AppError::BadRequest);
        }

        // 4. Create the company
        let new_company = company::ActiveModel {
            name: Set(payload.name),
            owner_id: Set(payload.owner_id),
            ..Default::default()
        };
        let new_company = new_company.insert(&txn).await?;

        // 5. Prepare the list of members for insertion (including the owner)
        let member_models = user_ids_to_check
            .into_iter()
            .map(|user_id| company_member::ActiveModel {
                company_id: Set(new_company.id),
                user_id: Set(user_id),
            })
            .collect::<Vec<_>>();

        CompanyMember::insert_many(member_models).exec(&txn).await?;

        // 6. Commit the transaction
        txn.commit().await?;

        // 7. Find the owner DTO from the list of found users
        let owner_dto = found_users
            .iter()
            .find(|u| u.id == new_company.owner_id)
            .cloned()
            .ok_or(AppError::InternalServerError)?; // Should not happen

        // 8. Construct the final response
        let response = CompanyDetailsResponse {
            id: new_company.id,
            name: new_company.name,
            owner: owner_dto,
            members: found_users,
        };

        Ok(response)
    }

    pub async fn get_all_companies(
        state: &AppState,
    ) -> Result<Vec<CompanyDetailsResponse>, AppError> {
        // 1. Fetch all companies
        let companies = Company::find().all(&state.conn).await?;
        if companies.is_empty() {
            return Ok(vec![]);
        }

        let company_ids: Vec<Uuid> = companies.iter().map(|c| c.id).collect();
        let owner_ids: Vec<Uuid> = companies.iter().map(|c| c.owner_id).collect();

        // 2. Fetch all members related to these companies
        let members_relations = CompanyMember::find()
            .filter(company_member::Column::CompanyId.is_in(company_ids.clone()))
            .all(&state.conn)
            .await?;

        let all_user_ids: Vec<Uuid> = members_relations
            .iter()
            .map(|m| m.user_id)
            .chain(owner_ids.into_iter())
            .collect::<HashSet<_>>() // Make unique
            .into_iter()
            .collect();

        // 3. Fetch all required user data in one go
        let all_users_dto: Vec<UserDto> = User::find()
            .filter(users::Column::Id.is_in(all_user_ids))
            .into_model::<UserDto>()
            .all(&state.conn)
            .await?;

        // 4. Create hashmaps for quick lookups
        let users_map: HashMap<Uuid, UserDto> =
            all_users_dto.into_iter().map(|u| (u.id, u)).collect();

        let mut members_map: HashMap<Uuid, Vec<UserDto>> = HashMap::new();
        for relation in members_relations {
            if let Some(user) = users_map.get(&relation.user_id) {
                members_map
                    .entry(relation.company_id)
                    .or_default()
                    .push(user.clone());
            }
        }

        // 5. Build the final response
        let mut response = Vec::new();
        for company in companies {
            let owner = users_map
                .get(&company.owner_id)
                .cloned()
                .ok_or(AppError::InternalServerError)?; // Should exist due to FK

            let members = members_map.remove(&company.id).unwrap_or_default();

            response.push(CompanyDetailsResponse {
                id: company.id,
                name: company.name,
                owner,
                members,
            });
        }

        Ok(response)
    }

    pub async fn get_company_by_id(
        state: &AppState,
        id: Uuid,
    ) -> Result<CompanyDetailsResponse, AppError> {
        let company = Company::find_by_id(id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let members_relations = CompanyMember::find()
            .filter(company_member::Column::CompanyId.eq(company.id))
            .all(&state.conn)
            .await?;

        let member_ids: Vec<Uuid> = members_relations.iter().map(|m| m.user_id).collect();

        // All unique users we need to fetch
        let mut all_user_ids: HashSet<Uuid> = member_ids.iter().cloned().collect();
        all_user_ids.insert(company.owner_id);

        let all_users: Vec<UserDto> = User::find()
            .filter(users::Column::Id.is_in(all_user_ids.into_iter().collect::<Vec<_>>()))
            .into_model::<UserDto>()
            .all(&state.conn)
            .await?;

        let users_map: HashMap<Uuid, UserDto> = all_users.into_iter().map(|u| (u.id, u)).collect();

        let owner = users_map
            .get(&company.owner_id)
            .cloned()
            .ok_or(AppError::InternalServerError)?; // Should not happen, FK constraint

        let members: Vec<UserDto> = member_ids
            .iter()
            .filter_map(|id| users_map.get(id).cloned())
            .collect();

        Ok(CompanyDetailsResponse {
            id: company.id,
            name: company.name,
            owner,
            members,
        })
    }
    
    pub async fn delete_company(state: &AppState, id: Uuid) -> Result<(), AppError> {
        let company = Company::find_by_id(id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        company.delete(&state.conn).await?;

        Ok(())
    }

    pub async fn update_company(
        state: &AppState,
        id: Uuid,
        payload: UpdateCompanyRequest,
    ) -> Result<CompanyDetailsResponse, AppError> {
        let txn = state.conn.begin().await?;

        let company = Company::find_by_id(id)
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;
        let mut company: company::ActiveModel = company.into_active_model();

        if let Some(name) = payload.name {
            if !name.is_empty() {
                company.name = Set(name);
            }
        }

        if let Some(owner_id) = payload.owner_id {
            User::find_by_id(owner_id)
                .one(&txn)
                .await?
                .ok_or(AppError::BadRequest)?;
            company.owner_id = Set(owner_id);
        }

        if let Some(members) = payload.members {
            let mut new_member_ids: HashSet<Uuid> = members.into_iter().collect();
            let owner_id = company.owner_id.as_ref().clone();
            new_member_ids.insert(owner_id);

            let found_users_count = User::find()
                .filter(users::Column::Id.is_in(new_member_ids.clone()))
                .count(&txn)
                .await?;
            if found_users_count != new_member_ids.len() as u64 {
                return Err(AppError::BadRequest);
            }

            CompanyMember::delete_many()
                .filter(company_member::Column::CompanyId.eq(id))
                .exec(&txn)
                .await?;

            let member_models = new_member_ids
                .into_iter()
                .map(|user_id| company_member::ActiveModel {
                    company_id: Set(id),
                    user_id: Set(user_id),
                })
                .collect::<Vec<_>>();
            CompanyMember::insert_many(member_models).exec(&txn).await?;
        }

        company.update(&txn).await?;

        txn.commit().await?;

        Self::get_company_by_id(state, id).await
    }
    
    pub async fn get_company_members(
        state: &AppState,
        id: Uuid,
    ) -> Result<Vec<UserDto>, AppError> {
        let company = Company::find_by_id(id)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let members: Vec<UserDto> = User::find()
            .filter(
                users::Column::Id.in_subquery(
                    Query::select()
                        .column(company_member::Column::UserId)
                        .from(CompanyMember::Table)
                        .and_where(company_member::Column::CompanyId.eq(company.id))
                        .to_owned(),
                ),
            )
            .into_model::<UserDto>()
            .all(&state.conn)
            .await?;
        
        Ok(members)
    }

    pub async fn get_my_companies(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Vec<CompanyDetailsResponse>, AppError> {
        let user_company_ids: Vec<Uuid> = CompanyMember::find()
            .select_only()
            .column(company_member::Column::CompanyId)
            .filter(company_member::Column::UserId.eq(user_id))
            .into_tuple()
            .all(&state.conn)
            .await?;

        if user_company_ids.is_empty() {
            return Ok(vec![]);
        }

        // Modified get_all_companies logic
        let companies = Company::find()
            .filter(company::Column::Id.is_in(user_company_ids))
            .all(&state.conn)
            .await?;
        
        let company_ids: Vec<Uuid> = companies.iter().map(|c| c.id).collect();
        let owner_ids: Vec<Uuid> = companies.iter().map(|c| c.owner_id).collect();

        let members_relations = CompanyMember::find()
            .filter(company_member::Column::CompanyId.is_in(company_ids.clone()))
            .all(&state.conn)
            .await?;

        let all_user_ids: Vec<Uuid> = members_relations
            .iter()
            .map(|m| m.user_id)
            .chain(owner_ids.into_iter())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let all_users_dto: Vec<UserDto> = User::find()
            .filter(users::Column::Id.is_in(all_user_ids))
            .into_model::<UserDto>()
            .all(&state.conn)
            .await?;

        let users_map: HashMap<Uuid, UserDto> =
            all_users_dto.into_iter().map(|u| (u.id, u)).collect();

        let mut members_map: HashMap<Uuid, Vec<UserDto>> = HashMap::new();
        for relation in members_relations {
            if let Some(user) = users_map.get(&relation.user_id) {
                members_map
                    .entry(relation.company_id)
                    .or_default()
                    .push(user.clone());
            }
        }

        let mut response = Vec::new();
        for company in companies {
            let owner = users_map
                .get(&company.owner_id)
                .cloned()
                .ok_or(AppError::InternalServerError)?;

            let members = members_map.remove(&company.id).unwrap_or_default();

            response.push(CompanyDetailsResponse {
                id: company.id,
                name: company.name,
                owner,
                members,
            });
        }

        Ok(response)
    }
}
