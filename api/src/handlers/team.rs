use crate::{
    AppState,
    dtos::{common::MessageResponse, team::{CreateTeamInvitation, CreateTeamMarketRequest, CreateTeamRequest, MarketTeamRequestDto, TeamDto, TeamInvitationDto, TeamMarketRequestDto, UpdateTeamRequest}, user::UserDto},
    error::AppError,
    services::team::TeamService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post, put},
};
use entity::{request_status::RequestStatus, role::Role};
use macros::has_any_role;
use sea_orm::prelude::Uuid;

pub fn team_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_teams).post(create_team).put(update_team))
        .route("/my/{idea_id}", get(get_my_teams))
        .route("/{id}", get(get_team_by_id).delete(delete_team))
        .route("/invitations", post(send_invites_to_users))
        .route("/invitations/my", get(get_team_invitations_by_user))
        .route("/invitations/team/{team_id}", get(get_team_invitations_by_team))
        .route("/invitations/status/{invitation_id}/{new_status}", put(update_team_invitation_status))
        .route("/requests/{team_id}", post(create_team_request))
        .route("/requests/team/{team_id}", get(get_team_requests_by_team))
        .route("/requests/status/{invitation_id}/{new_status}", put(update_team_request_status))
        .route("/market/requests/{team_id}", get(get_team_market_requests))
        .route("/market/request", post(create_market_request))
        .route("/market/request/{idea_market_id}", get(get_market_requests_by_idea_market))
        .route("/market/request/status/{request_id}/{new_status}", put(update_market_request_status))
        .route("/market/request/{request_id}", delete(delete_annulled_market_request))
        .route("/market/accept/{idea_market_id}/{team_id}", put(accept_for_idea_market))
        .route("/market/{market_id}", put(set_market_id))
        .route("/members/add/{team_id}/{user_id}", post(add_team_member))
        .route("/members/kick/{team_id}/{user_id}", delete(kick))
        .route("/leave/{team_id}", delete(leave))
        .route("/leader/{team_id}/{user_id}", put(update_team_leader))
}

async fn get_teams(State(state): State<AppState>, claims: Claims) -> Json<Vec<TeamDto>> {
    Json(TeamService::get_all(&state, claims.sub).await)
}
async fn get_my_teams(
    State(state): State<AppState>,
    claims: Claims,
    Path(idea_id): Path<Uuid>,
) -> Json<Vec<TeamDto>> {
    Json(TeamService::get_all_my(&state, claims.sub, idea_id).await)
}
async fn get_team_by_id(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<TeamDto, AppError> {
    let team = TeamService::get_one(&state, id, claims.sub).await?;
    Ok(team)
}
#[has_any_role(Admin, TeamOwner)]
async fn create_team(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateTeamRequest>,
) -> Result<TeamDto, AppError> {
    let team = TeamService::create(&state, payload, claims.sub).await?;
    Ok(team)
}
#[has_any_role(Admin, TeamOwner, TeamLeader)]
async fn update_team(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateTeamRequest>,
) -> Result<TeamDto, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    let team = TeamService::update(&state, payload, claims.sub, is_admin).await?;
    Ok(team)
}

async fn get_team_invitations_by_user(
    State(state): State<AppState>,
    claims: Claims
) -> Json<Vec<TeamInvitationDto>> {
    let invitations = TeamService::get_team_invitations_by_user(&state, claims.sub).await;
    Json(invitations)
}

async fn get_team_invitations_by_team(
    State(state): State<AppState>,
    _: Claims,
    Path(team_id): Path<Uuid>,
) -> Json<Vec<TeamInvitationDto>> {
    let invitations = TeamService::get_team_invitations_by_team(&state, team_id, false).await;
    Json(invitations)
}

async fn get_team_requests_by_team(
    State(state): State<AppState>,
    _: Claims,
    Path(team_id): Path<Uuid>,
) -> Json<Vec<TeamInvitationDto>> {
    let invitations = TeamService::get_team_invitations_by_team(&state, team_id, true).await;
    Json(invitations)
}

async fn get_team_market_requests(
    State(state): State<AppState>,
    _: Claims,
    Path(team_id): Path<Uuid>,
) -> Json<Vec<TeamMarketRequestDto>> {
    let team_market_requests = TeamService::get_team_market_requests(&state, team_id).await;
    Json(team_market_requests)
}

async fn get_market_requests_by_idea_market(
    State(state): State<AppState>,
    _: Claims,
    Path(idea_market_id): Path<Uuid>,
) -> Result<Json<Vec<MarketTeamRequestDto>>, AppError> {
    let team_market_requests =
        TeamService::get_market_requests_by_idea_market(&state, idea_market_id).await?;
    Ok(Json(team_market_requests))
}

#[has_any_role(Admin, TeamOwner, TeamLeader)]
async fn send_invites_to_users(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<Vec<CreateTeamInvitation>>,
) -> Result<(), AppError> {
    TeamService::send_invites_to_users(&state, payload, claims).await
}

#[has_any_role(Admin, Member)]
async fn create_team_request(
    State(state): State<AppState>,
    claims: Claims,
    Path(team_id): Path<Uuid>,
) -> Result<(), AppError> {
    TeamService::create_team_request(&state, team_id, claims).await
}

#[has_any_role(Admin, TeamOwner, TeamLeader)]
async fn create_market_request(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateTeamMarketRequest>,
) -> Result<TeamMarketRequestDto, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    let request = TeamService::create_market_request(&state, payload, claims.sub, is_admin).await?;
    Ok(request)
}

#[has_any_role(Admin, TeamOwner, TeamLeader)]
async fn add_team_member(
    State(state): State<AppState>,
    claims: Claims,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<UserDto, AppError> {
    let member = TeamService::add_team_member(&state, team_id, user_id).await?;
    Ok(member)
}

#[has_any_role(Admin, TeamOwner)]
async fn delete_team(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    TeamService::delete(&state, id, claims.sub, is_admin).await
}

#[has_any_role(Admin, TeamOwner, TeamLeader)]
async fn kick(
    State(state): State<AppState>,
    claims: Claims,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    TeamService::kick(&state, team_id, user_id).await
}

#[has_any_role(Admin, Member, TeamOwner, TeamLeader)]
async fn leave(
    State(state): State<AppState>,
    claims: Claims,
    Path(team_id): Path<Uuid>,
) -> Result<(), AppError> {
    TeamService::leave(&state, team_id, claims.sub).await
}

#[has_any_role(Admin, ProjectOffice)]
async fn set_market_id(
    State(state): State<AppState>,
    claims: Claims,
    Path(market_id): Path<Uuid>,
    Json(payload): Json<Vec<Uuid>>,
) -> Result<(), AppError> {
    TeamService::set_market_id(&state, payload, market_id).await
}

#[has_any_role(Admin, TeamOwner, TeamLeader, Initiator)]
async fn update_market_request_status(
    State(state): State<AppState>,
    claims: Claims,
    Path((request_id, new_status)): Path<(Uuid, RequestStatus)>,
) -> Result<MessageResponse, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    TeamService::update_market_request_status(&state, request_id, new_status, claims.sub, is_admin)
        .await?;
    Ok(MessageResponse {
        message: "Статус заявки обновлен".to_string(),
    })
}

async fn delete_annulled_market_request(
    State(state): State<AppState>,
    _: Claims,
    Path(request_id): Path<Uuid>,
) -> Result<MessageResponse, AppError> {
    TeamService::delete_annulled_market_request(&state, request_id).await?;
    Ok(MessageResponse {
        message: "Заявка удалена".to_string(),
    })
}

#[has_any_role(Admin, Initiator)]
async fn accept_for_idea_market(
    State(state): State<AppState>,
    claims: Claims,
    Path((idea_market_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<TeamDto, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    let team = TeamService::accept_for_idea_market(&state, idea_market_id, team_id, claims.sub, is_admin).await?;
    Ok(team)
}

#[has_any_role(Admin, TeamOwner)]
async fn update_team_leader(
    State(state): State<AppState>,
    claims: Claims,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    TeamService::update_team_leader(&state, team_id, user_id, is_admin).await
}

#[has_any_role(Admin, TeamOwner, TeamLeader, Member)]
async fn update_team_invitation_status(
    State(state): State<AppState>,
    claims: Claims,
    Path((invitation_id, new_status)): Path<(Uuid, RequestStatus)>,
) -> Result<TeamInvitationDto, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    let invitation = TeamService::update_team_invitation_status(&state, invitation_id, new_status, claims.sub, is_admin).await?;
    Ok(invitation)
}

#[has_any_role(Admin, TeamOwner, TeamLeader, Member)]
async fn update_team_request_status(
    State(state): State<AppState>,
    claims: Claims,
    Path((invitation_id, new_status)): Path<(Uuid, RequestStatus)>,
) -> Result<TeamInvitationDto, AppError> {
    let is_admin = claims.roles.contains(&Role::Admin);
    let invitation = TeamService::update_team_request_status(&state, invitation_id, new_status, claims.sub, is_admin).await?;
    Ok(invitation)
}
