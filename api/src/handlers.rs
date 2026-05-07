use crate::{AppState, config::GLOBAL_CONFIG};
use axum::Router;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub mod auth;
pub mod company;
pub mod group;
pub mod idea;
pub mod idea_market;
pub mod invitation;
pub mod market;
pub mod profile;
pub mod project;
pub mod rating;
pub mod skill;
pub mod team;
pub mod user;
pub mod sprint;
pub mod tag;
pub mod task;
pub mod task_movement_log;

pub fn main_router() -> Router<AppState> {
    let avatar_dir = PathBuf::from(GLOBAL_CONFIG.avatar_path.clone());

    Router::new()
        .nest("/auth", auth::auth_router())
        .nest("/invitation", invitation::invitation_router())
        .nest("/company", company::company_router())
        .nest("/profile", profile::profile_router())
        .nest("/users", user::user_router())
        .nest("/skill", skill::skill_router())
        .nest("/group", group::group_router())
        .nest("/idea", idea::idea_router())
        .nest("/idea_market", idea_market::idea_market_router())
        .nest("/rating", rating::rating_router())
        .nest("/market", market::market_router())
        .nest("/project", project::project_router())
        .nest("/sprint", sprint::sprint_router())
        .nest("/tag", tag::tag_router())
        .nest("/task", task::task_router())
        .nest("/log", task_movement_log::task_movement_log_router())
        .nest("/team", team::team_router())
        .nest_service("/images/avatar", ServeDir::new(avatar_dir))
}
