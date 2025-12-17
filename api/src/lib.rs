use crate::{
    config::GLOBAL_CONFIG, handlers::main_router, utils::startup::create_admin,
    workers::invitation_worker,
};
use axum::Router;
use axum::http::{HeaderValue, Method, header};
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tower_http::cors::CorsLayer;

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

    let cors = CorsLayer::new()
        .allow_origin(
            GLOBAL_CONFIG
                .client_url
                .clone()
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_credentials(true)
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .nest("/api", main_router())
        .with_state(state)
        .layer(cors);

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
