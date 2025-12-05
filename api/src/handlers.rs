use axum::Router;

use crate::{AppState, handlers::{admin::admin_router, auth::auth_router}};

pub mod auth;
pub mod admin;

pub fn main_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_router())
        .nest("/admin", admin_router())
}