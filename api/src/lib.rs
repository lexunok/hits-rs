use std::env;

use axum::Router;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

use crate::handlers::main_router;

mod error;
mod handlers;
mod models;
mod utils;
mod workers;

#[tokio::main]
pub async fn start() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL")?;
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or("redis://127.0.0.1/".to_string());

    let conn = Database::connect(db_url).await?;
    Migrator::up(&conn, None).await?;
    let redis_client = redis::Client::open(redis_url)?;

    let state = AppState { conn, redis_client };

    let redis_clone = state.redis_client.clone();
    tokio::spawn(async move {
        workers::invitation_worker(redis_clone).await;
    });

    let app = Router::new().nest("/api", main_router()).with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
    redis_client: redis::Client,
}
