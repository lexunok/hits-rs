use crate::{
    AppState,
    dtos::{
        auth::EmailResetPayload,
        common::{IdResponse, MessageResponse},
        profile::{ProfileUpdatePayload, UserDto},
    },
    error::AppError,
    services::user::UserService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put},
};
use sea_orm::prelude::Uuid;

pub fn profile_router() -> Router<AppState> {
    Router::new()
        .route("/users", get(get_all_users))
        .route("/users/:id", get(get_user))
        .route("/users", put(update_profile))
        .route(
            "/email/verification/:new_email",
            post(request_to_update_email),
        )
        .route("/email/:id", put(confirm_and_update_email))
}

async fn get_all_users(State(state): State<AppState>, _: Claims) -> Json<Vec<UserDto>> {
    Json(UserService::get_users(&state).await)
}

async fn get_user(
    State(state): State<AppState>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Result<UserDto, AppError> {
    UserService::get_user(&state, id).await
}

async fn update_profile(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<ProfileUpdatePayload>,
) -> Result<MessageResponse, AppError> {
    UserService::update_profile(&state, payload, claims.sub).await?;

    Ok(MessageResponse {
        message: "Успешное обновление профиля".to_string(),
    })
}

async fn request_to_update_email(
    State(state): State<AppState>,
    _: Claims,
    Path(new_email): Path<String>,
) -> Result<IdResponse, AppError> {
    let verification_id = UserService::request_email_change(&state, new_email).await?;

    Ok(IdResponse {
        id: verification_id,
    })
}

async fn confirm_and_update_email(
    State(state): State<AppState>,
    claims: Claims,
    payload: axum::Json<EmailResetPayload>,
) -> Result<MessageResponse, AppError> {
    UserService::confirm_email_change(&state, claims, payload.0).await?;

    Ok(MessageResponse {
        message: "Успешное обновление почты".to_string(),
    })
}

// #[has_any_role(Admin, TeamOwner)]
// async fn change_team_leader(
//     State(state): State<AppState>,
//     claims: Claims,
//     Path(old_team_leader_id): Path<Uuid>,
//     Path(new_team_leader_id): Path<Uuid>,
// ) -> Result<impl IntoResponse, GlobalError> {

//     Ok(Json(CustomMessage {
//         message: "Успешное обновление профиля".to_string(),
//     }))
// }
// public void changeTeamLeader(String teamLeaderId, String userId){
//     User oldTeamLeader = userRepository.findById(teamLeaderId).orElseThrow(() -> new NotFoundException("Not found"));
//     oldTeamLeader.getRoles().remove(Role.TEAM_LEADER);
//     profileClient.checkUser(mapper.map(userRepository.save(oldTeamLeader), UserDTO.class));
//     template.opsForHash().delete("user", oldTeamLeader.getEmail().toLowerCase());
//     User newTeamLeader = userRepository.findById(userId).orElseThrow(() -> new NotFoundException("Not found"));
//     if (newTeamLeader.getRoles().stream().noneMatch(role -> role.equals(Role.TEAM_LEADER))) {
//         newTeamLeader.getRoles().add(Role.TEAM_LEADER);
//         profileClient.checkUser(mapper.map(userRepository.save(newTeamLeader), UserDTO.class));
//         template.opsForHash().delete("user", newTeamLeader.getEmail().toLowerCase());
//     }
// }
