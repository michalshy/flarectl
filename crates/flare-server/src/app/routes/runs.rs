use crate::app::auth;
use crate::app::error::AppError;
use axum::routing::{get, post};
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
};
use sqlx::query_as;
use uuid::Uuid;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/runs/{id}", post(put_runs))
        .route("/runs/{id}/heartbeat", post(post_heartbeat))
        .route("/jobs/{id}/finish", post(post_finish))
}

async fn put_runs(State(state): State<AppState>) -> Result<(), AppError> {
    Ok(())
}

async fn post_heartbeat(State(state): State<AppState>) -> Result<(), AppError> {
    Ok(())
}

async fn post_finish(State(state): State<AppState>) -> Result<(), AppError> {
    Ok(())
}
