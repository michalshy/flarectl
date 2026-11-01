mod auth;
mod db;
pub mod error;
mod routes;
mod sweeper;

use std::time::Duration;

use axum::{Router, middleware, routing::get};
use sqlx::{PgPool, postgres::PgPoolOptions};
use sweeper::sweeper;

use crate::app::auth::{AuthAdmin, AuthJob};

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    admin_token: String,
}

impl AppState {
    pub async fn connect(db_url: &str, admin_token: String) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(db_url)
            .await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool, admin_token })
    }
}

pub async fn serve(
    state: AppState,
    addr: std::net::SocketAddr,
    heartbeat_timeout: Duration,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tokio::spawn(sweeper(state.clone(), heartbeat_timeout));
    axum::serve(listener, router(state)).await?;
    Ok(())
}

fn router(state: AppState) -> Router {
    let v1 = Router::new()
        .merge(routes::jobs::router())
        .merge(routes::workers::router())
        .route_layer(middleware::from_extractor_with_state::<AuthAdmin, _>(
            state.clone(),
        ))
        .merge(routes::runs::router())
        .merge(routes::steps::router())
        .route_layer(middleware::from_extractor_with_state::<AuthJob, _>(
            state.clone(),
        ))
        .route("/health", get(routes::health));

    Router::new().nest("/v1", v1).with_state(state)
}
