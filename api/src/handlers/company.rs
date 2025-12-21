use crate::{
    AppState,
    dtos::{
        common::MessageResponse,
        company::{CompanyResponse, CreateCompanyRequest, UpdateCompanyRequest},
        profile::UserDto,
    },
    error::AppError,
    services::company::CompanyService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use macros::has_role;
use sea_orm::prelude::Uuid;

pub fn company_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(get_all_companies)
                .post(create_company)
                .put(update_company),
        )
        .route("/{id}", get(get_company_by_id).delete(delete_company))
        .route("/{id}/members", get(get_company_members))
        .route("/my", get(get_my_companies))
}
async fn get_all_companies(State(state): State<AppState>, _: Claims) -> Json<Vec<CompanyResponse>> {
    let companies = CompanyService::get_all(&state).await;
    Json(companies)
}
async fn get_company_members(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Json<Vec<UserDto>> {
    let members = CompanyService::get_members(&state, id).await;
    Json(members)
}
async fn get_my_companies(
    State(state): State<AppState>,
    claims: Claims,
) -> Json<Vec<CompanyResponse>> {
    let companies = CompanyService::get_my(&state, claims.sub).await;
    Json(companies)
}
async fn get_company_by_id(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Result<CompanyResponse, AppError> {
    let company = CompanyService::get_one(&state, id).await?;
    Ok(company)
}

#[has_role(Admin)]
async fn create_company(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateCompanyRequest>,
) -> Result<CompanyResponse, AppError> {
    let company = CompanyService::create(&state, payload).await?;
    Ok(company)
}
#[has_role(Admin)]
async fn update_company(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateCompanyRequest>,
) -> Result<CompanyResponse, AppError> {
    let company = CompanyService::update(&state, payload).await?;
    Ok(company)
}

#[has_role(Admin)]
async fn delete_company(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    CompanyService::delete(&state, id).await?;
    Ok(MessageResponse {
        message: "Компания успешно удалена".to_string(),
    })
}
