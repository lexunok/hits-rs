use crate::{
    dtos::{
        common::MessageResponse,
        company::{CompanyDetailsResponse, CreateCompanyRequest, UpdateCompanyRequest},
        profile::UserDto,
    },
    error::AppError,
    services::company::CompanyService,
    utils::security::Claims,
    AppState,
};
use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use macros::has_role;
use sea_orm::prelude::Uuid;

pub fn company_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_company).get(get_all_companies))
        .route("/my", get(get_my_companies))
        .route(
            "/:id",
            get(get_company_by_id)
                .delete(delete_company)
                .put(update_company),
        )
        .route("/:id/members", get(get_company_members))
}

#[has_role(Admin)]
async fn create_company(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateCompanyRequest>,
) -> Result<Json<CompanyDetailsResponse>, AppError> {
    let company = CompanyService::create_company(&state, payload).await?;
    Ok(Json(company))
}

async fn get_all_companies(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<CompanyDetailsResponse>>, AppError> {
    let companies = CompanyService::get_all_companies(&state).await?;
    Ok(Json(companies))
}

async fn get_my_companies(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<CompanyDetailsResponse>>, AppError> {
    let companies = CompanyService::get_my_companies(&state, claims.sub).await?;
    Ok(Json(companies))
}

async fn get_company_by_id(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<CompanyDetailsResponse>, AppError> {
    let company = CompanyService::get_company_by_id(&state, id).await?;
    Ok(Json(company))
}

#[has_role(Admin)]
async fn delete_company(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    CompanyService::delete_company(&state, id).await?;
    Ok(MessageResponse {
        message: "Компания успешно удалена".to_string(),
    })
}

#[has_role(Admin)]
async fn update_company(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCompanyRequest>,
) -> Result<Json<CompanyDetailsResponse>, AppError> {
    let company = CompanyService::update_company(&state, id, payload).await?;
    Ok(Json(company))
}

async fn get_company_members(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<UserDto>>, AppError> {
    let members = CompanyService::get_company_members(&state, id).await?;
    Ok(Json(members))
}
