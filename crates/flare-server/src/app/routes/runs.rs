use crate::app::auth::AuthJob;
use crate::app::error::AppError;
use axum::routing::{post, put};
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::event::{EventKind, EventSource};
use flare_types::run::{FinishRun, RunFailure, RunStatus, StartRun};
use flare_types::step::StepStatus;
use uuid::Uuid;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/runs/{id}", put(put_run))
        .route("/runs/{id}/heartbeat", post(post_heartbeat))
        .route("/runs/{id}/finish", post(post_finish))
}

async fn put_run(
    State(state): State<AppState>,
    job: AuthJob,
    Path(id): Path<Uuid>,
    Json(input): Json<StartRun>,
) -> Result<StatusCode, AppError> {
    let mut tx = state.pool.begin().await?;

    let value = sqlx::query_scalar!(
        "INSERT INTO runs (id, job_id, worker_id, trigger, status)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE SET id = runs.id
        WHERE runs.job_id = EXCLUDED.job_id AND runs.status = $5
        RETURNING id",
        id,
        job.id,
        input.worker_id,
        input.trigger as _,
        RunStatus::Running as _
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(_) = value else {
        return Err(AppError::Conflict);
    };

    sqlx::query!(
        "INSERT INTO events (run_id, kind, source) VALUES ($1, $2, $3)",
        id,
        EventKind::Started as _,
        EventSource::Server as _
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(StatusCode::CREATED)
}

async fn post_heartbeat(
    State(state): State<AppState>,
    job: AuthJob,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let req = sqlx::query!(
        "UPDATE runs SET last_heartbeat_at = now() WHERE id = $1 AND job_id = $2 AND status = $3",
        id,
        job.id,
        RunStatus::Running as _
    )
    .execute(&state.pool)
    .await?;

    if req.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn post_finish(
    State(state): State<AppState>,
    job: AuthJob,
    Path(id): Path<Uuid>,
    Json(input): Json<FinishRun>,
) -> Result<StatusCode, AppError> {
    if input.final_status == RunStatus::Running {
        return Err(AppError::Validation("Can not finish with running status"));
    }
    let reason = (input.final_status == RunStatus::Failed).then_some(RunFailure::Reported);
    let payload = serde_json::json!({
        "status": input.final_status,
        "exit_code": input.exit_code,
    });

    let mut tx = state.pool.begin().await?;

    let updated = sqlx::query!(
        "UPDATE runs
        SET status = $4, failure_reason = $5, exit_code = $6, finished_at = now()
        WHERE id = $1 AND job_id = $2 AND status = $3
        RETURNING id",
        id,
        job.id,
        RunStatus::Running as _,
        input.final_status as _,
        reason as _,
        input.exit_code
    )
    .fetch_optional(&mut *tx)
    .await?;

    if updated.is_some() {
        sqlx::query!(
            "INSERT INTO events (run_id, kind, source, payload) VALUES ($1, $2, $3, $4)",
            id,
            EventKind::Finished as _,
            EventSource::Server as _,
            payload
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE steps SET status = $1, finished_at = now()
            WHERE run_id = $2 AND status = $3",
            StepStatus::Failed as _,
            id,
            StepStatus::Running as _
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    let exists = sqlx::query!(
        "SELECT status FROM runs WHERE id = $1 AND job_id = $2",
        id,
        job.id
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(_) = exists else {
        return Err(AppError::NotFound);
    };

    sqlx::query!(
        "INSERT INTO events (run_id, kind, source, payload) VALUES ($1, $2, $3, $4)",
        id,
        EventKind::LateReport as _,
        EventSource::Server as _,
        payload
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Err(AppError::Conflict)
}
