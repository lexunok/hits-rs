use std::path::PathBuf;

use crate::{
    AppState,
    config::GLOBAL_CONFIG,
    handlers::{
        auth::auth_router, company::company_router, invitation::invitation_router,
        profile::profile_router, user::user_router,
    },
};
use axum::Router;
use tower_http::services::ServeDir;

pub mod auth;
pub mod company;
pub mod invitation;
pub mod profile;
pub mod user;

pub fn main_router() -> Router<AppState> {
    let avatar_dir = PathBuf::from(GLOBAL_CONFIG.avatar_path.clone());

    Router::new()
        .nest("/auth", auth_router())
        .nest("/invitation", invitation_router())
        .nest("/company", company_router())
        .nest("/profile", profile_router())
        .nest("/users", user_router())
        .nest_service("/images/avatar", ServeDir::new(avatar_dir))
}
