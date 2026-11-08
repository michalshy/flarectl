use crate::app::auth::AuthJob;
use crate::app::error::AppError;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::error::FlareErrorResponse;
use flare_types::event::{EventKind, EventSource};
use flare_types::step::{CreateStep, FinishStep, StepStatus};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::app::{AppState, db};

pub fn job_api() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(put_step))
        .routes(routes!(post_finish))
}

/// Start a step of a running run. Repeating it for the same step succeeds.
#[utoipa::path(
    put,
    path = "/runs/{run_id}/steps/{step_id}",
    operation_id = "start_step",
    tag = "steps",
    params(
        ("run_id" = Uuid, Path, description = "Run id"),
        ("step_id" = Uuid, Path, description = "Step id"),
    ),
    request_body = CreateStep,
    responses(
        (status = CREATED, description = "Step started"),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such run for this job", body = FlareErrorResponse),
        (status = CONFLICT, description = "Run is not running or step id belongs to another run", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
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

    if !db::runs::run_exists(&mut tx, run_id, Some(job.id)).await? {
        return Err(AppError::NotFound);
    };

    Err(AppError::Conflict)
}

/// Finish a step with its final status.
#[utoipa::path(
    post,
    path = "/runs/{run_id}/steps/{step_id}/finish",
    operation_id = "finish_step",
    tag = "steps",
    params(
        ("run_id" = Uuid, Path, description = "Run id"),
        ("step_id" = Uuid, Path, description = "Step id"),
    ),
    request_body = FinishStep,
    responses(
        (status = NO_CONTENT, description = "Step finished"),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such step for this job", body = FlareErrorResponse),
        (status = CONFLICT, description = "Step already finished", body = FlareErrorResponse),
        (status = UNPROCESSABLE_ENTITY, description = "Final status can not be running", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
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
