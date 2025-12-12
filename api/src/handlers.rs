use crate::{
    AppState,
    handlers::{admin::admin_router, auth::auth_router, profile::profile_router},
};
use axum::Router;

pub mod admin;
pub mod auth;
pub mod profile;

pub fn main_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_router())
        .nest("/admin", admin_router())
        .nest("/profile", profile_router())
}
