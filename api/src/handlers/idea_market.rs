use axum::{routing::get, Router};

use crate::AppState;

pub fn idea_market_router() -> Router<AppState> {
    Router::new()
}