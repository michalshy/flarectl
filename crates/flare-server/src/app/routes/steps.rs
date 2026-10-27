use crate::app::auth::AuthJob;
use crate::app::error::AppError;
use axum::routing::{post, put};
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::event::{EventKind, EventSource};
use flare_types::step::{CreateStep, FinishStep, StepStatus};
use uuid::Uuid;

use crate::app::{AppState, db};

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

    let started = db::steps::start_step(&mut tx, step_id, run_id, job.id, input.clone()).await?;

    if started.is_some() {
        let payload = serde_json::json!({
            "step_id": step_id,
            "name": input.name,
            "position": input.position,
        });

        db::events::insert_event(
            &mut tx,
            run_id,
            EventKind::StepStarted,
            EventSource::Server,
            payload,
        )
        .await?;
        tx.commit().await?;
        return Ok(StatusCode::CREATED);
    }

    if !db::runs::run_exists(&mut tx, run_id, job.id).await? {
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

    let finished = db::steps::finish_step(&mut tx, step_id, run_id, job.id, input).await?;

    if finished.is_some() {
        db::events::insert_event(
            &mut tx,
            run_id,
            EventKind::StepFinished,
            EventSource::Server,
            payload,
        )
        .await?;

        tx.commit().await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    if !db::steps::step_exists(&mut tx, step_id, run_id, job.id).await? {
        return Err(AppError::NotFound);
    };

    Err(AppError::Conflict)
}
