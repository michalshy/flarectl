use crate::app::error::AppError;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::worker::{RegisterWorker, Worker};
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
    let worker = sqlx::query_as!(
        Worker,
        "SELECT id, name, version, labels, registered_at, last_seen_at FROM workers WHERE id = $1",
        id
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(worker))
}

pub async fn put_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<RegisterWorker>,
) -> Result<Json<Worker>, AppError> {
    let inserted = sqlx::query_as!(
        Worker,
        "
            INSERT INTO workers (id, name, version)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                version = EXCLUDED.version,
                last_seen_at = now()
            RETURNING id, name, version, labels, registered_at, last_seen_at
        ",
        id,
        input.name,
        input.version,
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(inserted))
}

pub async fn heartbeat(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let updated = sqlx::query!("UPDATE workers SET last_seen_at = now() WHERE id = $1", id)
        .execute(&state.pool)
        .await?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
