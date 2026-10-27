use crate::app::error::AppError;
use axum::routing::{get, post, put};
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::worker::{RegisterWorker, Worker};
use uuid::Uuid;

use crate::app::{AppState, db};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/workers", get(get_workers))
        .route("/workers/{id}", put(put_worker).get(get_worker))
        .route("/workers/{id}/heartbeat", post(post_heartbeat))
}

async fn get_workers(State(state): State<AppState>) -> Result<Json<Vec<Worker>>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let workers = db::workers::get_workers(&mut conn).await?;
    Ok(Json(workers))
}

async fn get_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Worker>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let worker = db::workers::get_worker(&mut conn, id).await?;
    Ok(Json(worker))
}

async fn put_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<RegisterWorker>,
) -> Result<Json<Worker>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let inserted = db::workers::insert_worker(&mut conn, id, input).await?;
    Ok(Json(inserted))
}

async fn post_heartbeat(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.pool.acquire().await?;
    if db::workers::touch_heartbeat(&mut conn, id).await? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
