use axum::Router;

use crate::{AppState, handlers::auth::auth_router};

pub mod auth;

pub fn main_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_router())
}