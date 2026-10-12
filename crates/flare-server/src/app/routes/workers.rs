use crate::app::error::AppError;
use axum::extract::{Json, Path, State};
use flare_types::worker::{RegisteredWorker, Worker};
use uuid::Uuid;

use crate::app::AppState;

pub async fn get(State(state): State<AppState>) -> Result<Json<Vec<Worker>>, AppError> {
    let workers = sqlx::query_as!(Worker, "SELECT * FROM workers")
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(workers))
}

pub async fn get_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Worker>, AppError> {
    let worker = sqlx::query_as!(Worker, "SELECT * FROM workers WHERE id = $1", id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(worker))
}

pub async fn put_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<RegisteredWorker>,
) -> &'static str {
    "ok"
}

pub async fn heartbeat(State(state): State<AppState>, Path(id): Path<Uuid>) -> &'static str {
    "ok"
}
