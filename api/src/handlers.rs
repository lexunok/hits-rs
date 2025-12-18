use crate::{
    AppState,
    handlers::{
        invitation::invitation_router, auth::auth_router, profile::profile_router,
        user::user_router,
    },
};
use axum::Router;

pub mod invitation;
pub mod auth;
// pub mod company;
pub mod profile;
pub mod user;

pub fn main_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_router())
        .nest("/invitation", invitation_router())
        // .nest("/company", company_router())
        .nest("/profile", profile_router())
        .nest("/users", user_router())
}
