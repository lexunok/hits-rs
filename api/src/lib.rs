use std::env;

use axum::Router;
use migration::{MigratorTrait, Migrator};
use sea_orm::{Database, DatabaseConnection};

use crate::handlers::main_router;

mod utils;
mod models;
mod handlers;
mod error;

#[tokio::main]
pub async fn start() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let port = env::var("PORT").unwrap_or("3000".to_string());

    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    Migrator::up(&conn, None).await.unwrap();

    let state = AppState { conn };

    let app = Router::new().nest("/api", main_router()).with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
}