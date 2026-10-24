mod auth;
mod db;
pub mod error;
mod routes;

use axum::{Router, routing::get};
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
}

impl AppState {
    pub async fn connect(db_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(db_url)
            .await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
}

pub async fn serve(state: AppState, addr: std::net::SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router(state)).await?;
    Ok(())
}

fn router(state: AppState) -> Router {
    let v1 = Router::new()
        .route("/health", get(routes::health))
        .merge(routes::workers::router())
        .merge(routes::jobs::router());

    Router::new().nest("/v1", v1).with_state(state)
}
