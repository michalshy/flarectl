use crate::app::auth::AuthJob;
use crate::app::error::AppError;
use axum::routing::{post, put};
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::event::{EventKind, EventSource};
use flare_types::run::RunStatus;
use flare_types::step::{CreateStep, FinishStep, StepStatus};
use uuid::Uuid;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/runs/{run_id}/steps/{step_id}", put(put_step))
        .route("/runs/{run_id}/steps/{step_id}/finish", post(post_finish))
}

pub async fn put_step(
    State(state): State<AppState>,
    job: AuthJob,
    Path((run_id, step_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<CreateStep>,
) -> Result<StatusCode, AppError> {
    let mut tx = state.pool.begin().await?;

    let started = sqlx::query_scalar!(
        "INSERT INTO steps (id, run_id, name, position, status)
        SELECT $1, $2, $3, $4, $5
        WHERE EXISTS (SELECT 1 FROM runs WHERE id = $2 AND job_id = $6 AND status = $7)
        ON CONFLICT (id) DO UPDATE SET id = steps.id WHERE steps.run_id = EXCLUDED.run_id
        RETURNING id",
        step_id,
        run_id,
        input.name,
        input.position,
        StepStatus::Running as _,
        job.id,
        RunStatus::Running as _,
    )
    .fetch_optional(&mut *tx)
    .await?;

    if started.is_some() {
        let payload = serde_json::json!({
            "step_id": step_id,
            "name": input.name,
            "position": input.position,
        });

        sqlx::query!(
            "INSERT INTO events (run_id, kind, source, payload) VALUES ($1, $2, $3, $4)",
            run_id,
            EventKind::StepStarted as _,
            EventSource::Server as _,
            payload,
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(StatusCode::CREATED);
    }

    let exists = sqlx::query_scalar!(
        "SELECT 1 FROM runs WHERE id = $1 AND job_id = $2",
        run_id,
        job.id
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(_) = exists else {
        return Err(AppError::NotFound);
    };

    Err(AppError::Conflict)
}

pub async fn post_finish(
    State(state): State<AppState>,
    job: AuthJob,
    Path((run_id, step_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<FinishStep>,
) -> Result<StatusCode, AppError> {
    if input.status == StepStatus::Running {
        return Err(AppError::Validation("Can not finish with running status"));
    }
    let payload = serde_json::json!({
        "step_id": step_id,
        "status": input.status,
    });

    let mut tx = state.pool.begin().await?;

    let updated = sqlx::query!(
        "UPDATE steps SET status = $1, finished_at = now()
        FROM runs
        WHERE steps.id = $2 AND steps.run_id = $3 AND steps.status = $4
          AND runs.id = steps.run_id AND runs.job_id = $5
        RETURNING steps.id",
        input.status as _,
        step_id,
        run_id,
        StepStatus::Running as _,
        job.id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    if updated.is_some() {
        sqlx::query!(
            "INSERT INTO events (run_id, kind, source, payload) VALUES ($1, $2, $3, $4)",
            run_id,
            EventKind::StepFinished as _,
            EventSource::Server as _,
            payload
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    let exists = sqlx::query_scalar!(
        "SELECT 1 FROM steps
        JOIN runs ON runs.id = steps.run_id
        WHERE steps.id = $1 AND steps.run_id = $2 AND runs.job_id = $3",
        step_id,
        run_id,
        job.id
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(_) = exists else {
        return Err(AppError::NotFound);
    };

    Err(AppError::Conflict)
}
