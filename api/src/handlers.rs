use axum::Router;

use crate::handlers::auth::auth_router;

pub mod auth;

pub fn main_router() -> Router {
    Router::new()
        .nest("/auth", auth_router())
}