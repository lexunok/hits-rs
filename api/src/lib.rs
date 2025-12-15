use crate::{
    config::GLOBAL_CONFIG, handlers::main_router, utils::startup::create_admin,
    workers::invitation_worker,
};
use axum::Router;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

mod config;
mod dtos;
mod error;
mod handlers;
mod services;
mod utils;
mod workers;

#[tokio::main]
pub async fn start() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenvy::dotenv().ok();

    let conn = Database::connect(GLOBAL_CONFIG.db_url.to_owned()).await?;
    Migrator::up(&conn, None).await?;
    create_admin(conn.clone()).await.unwrap();

    let redis_client = redis::Client::open(GLOBAL_CONFIG.redis_url.to_owned())?;

    let state = AppState { conn, redis_client };

    let redis_clone = state.redis_client.clone();
    tokio::spawn(async move {
        invitation_worker::invitation_worker(redis_clone).await;
    });

    let app = Router::new().nest("/api", main_router()).with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", GLOBAL_CONFIG.port)).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
    redis_client: redis::Client,
}
