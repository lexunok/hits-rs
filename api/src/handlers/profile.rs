use crate::{
    AppState,
    dtos::{
        auth::EmailResetPayload,
        common::{IdResponse, MessageResponse},
        profile::ProfileUpdatePayload,
    },
    error::AppError,
    services::user::UserService,
    utils::security::Claims,
};
use axum::{
    extract::{Multipart, Path, State},
    Json, Router,
    routing::{post, put},
};

pub fn profile_router() -> Router<AppState> {
    Router::new()
        .route("/", put(update_profile))
        .route("/avatar", post(upload_avatar))
        .route(
            "/email/verification/{new_email}",
            post(request_to_update_email),
        )
        .route("/email", put(confirm_and_update_email))
}

async fn upload_avatar(
    claims: Claims,
    mut multipart: Multipart,
) -> Result<MessageResponse, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|_| AppError::BadRequest)? {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "avatar" {
            let bytes = field.bytes().await.map_err(|_| AppError::BadRequest)?;
            UserService::upload_avatar(claims.sub, bytes).await?;

            return Ok(MessageResponse {
                message: "Аватар успешно обновлен".to_string(),
            });
        }
    }

    Err(AppError::BadRequest)
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
    Json(payload): Json<EmailResetPayload>,
) -> Result<MessageResponse, AppError> {
    UserService::confirm_email_change(&state, claims, payload).await?;

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
