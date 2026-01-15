use crate::{
    AppState,
    config::GLOBAL_CONFIG,
    handlers::{
        auth::auth_router, company::company_router, group::group_router, idea::idea_router,
        invitation::invitation_router, market::market_router, profile::profile_router,
        rating::rating_router, skill::skill_router, user::user_router,
    },
};
use axum::Router;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub mod auth;
pub mod company;
pub mod group;
pub mod idea;
pub mod invitation;
pub mod market;
pub mod profile;
pub mod rating;
pub mod skill;
pub mod user;

pub fn main_router() -> Router<AppState> {
    let avatar_dir = PathBuf::from(GLOBAL_CONFIG.avatar_path.clone());

    Router::new()
        .nest("/auth", auth_router())
        .nest("/invitation", invitation_router())
        .nest("/company", company_router())
        .nest("/profile", profile_router())
        .nest("/users", user_router())
        .nest("/skill", skill_router())
        .nest("/group", group_router())
        .nest("/idea", idea_router())
        .nest("/rating", rating_router())
        .nest("/market", market_router())
        .nest_service("/images/avatar", ServeDir::new(avatar_dir))
}
