use crate::app::auth::AuthJob;
use crate::app::db::events::select_events;
use crate::app::error::AppError;
use axum::extract::Query;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::error::FlareErrorResponse;
use flare_types::event::{Event, EventKind, EventSource};
use flare_types::run::{FinishRun, ListRuns, Run, RunDetail, RunFailure, RunStatus, StartRun};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::app::{AppState, db};

pub fn job_api() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(put_run))
        .routes(routes!(post_heartbeat))
        .routes(routes!(post_finish))
}

pub fn admin_api() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_runs))
        .routes(routes!(get_run))
        .routes(routes!(get_events))
}

/// Start a run with a client-chosen id. Repeating it for a running run of the same job succeeds.
#[utoipa::path(
    put,
    path = "/runs/{id}",
    operation_id = "start_run",
    tag = "runs",
    params(
        ("id" = Uuid, Path, description = "Run id"),
    ),
    request_body = StartRun,
    responses(
        (status = CREATED, description = "Run started"),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = CONFLICT, description = "Run id belongs to another job or the run is no longer running", body = FlareErrorResponse),
        (status = UNPROCESSABLE_ENTITY, description = "Referenced worker does not exist", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn put_run(
    State(state): State<AppState>,
    job: AuthJob,
    Path(id): Path<Uuid>,
    Json(input): Json<StartRun>,
) -> Result<StatusCode, AppError> {
    let mut tx = state.pool.begin().await?;

    let new_run = db::runs::start_run(&mut tx, id, job.id, input).await?;
    let Some(_) = new_run else {
        return Err(AppError::Conflict);
    };

    db::events::insert_event(
        &mut tx,
        id,
        EventKind::Started,
        EventSource::Server,
        serde_json::json!({}),
    )
    .await?;

    tx.commit().await?;
    Ok(StatusCode::CREATED)
}

/// Record a heartbeat for a running run.
#[utoipa::path(
    post,
    path = "/runs/{id}/heartbeat",
    operation_id = "heartbeat_run",
    tag = "runs",
    params(
        ("id" = Uuid, Path, description = "Run id"),
    ),
    responses(
        (status = NO_CONTENT, description = "Heartbeat recorded"),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No running run with this id for this job", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn post_heartbeat(
    State(state): State<AppState>,
    job: AuthJob,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.pool.acquire().await?;
    let rows = db::runs::touch_heartbeat(&mut conn, id, job.id).await?;

    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Finish a run with its final status.
#[utoipa::path(
    post,
    path = "/runs/{id}/finish",
    operation_id = "finish_run",
    tag = "runs",
    params(
        ("id" = Uuid, Path, description = "Run id"),
    ),
    request_body = FinishRun,
    responses(
        (status = NO_CONTENT, description = "Run finished"),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such run for this job", body = FlareErrorResponse),
        (status = CONFLICT, description = "Run already finished; recorded as a late report", body = FlareErrorResponse),
        (status = UNPROCESSABLE_ENTITY, description = "Final status can not be running", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
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

    let finished = db::runs::finish_run(&mut tx, id, job.id, input, reason).await?;

    if let Some(finished) = finished {
        db::events::insert_event(
            &mut tx,
            id,
            EventKind::Finished,
            EventSource::Server,
            payload,
        )
        .await?;

        db::steps::fail_running_steps(&mut tx, finished).await?;

        tx.commit().await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    if !db::runs::run_exists(&mut tx, id, Some(job.id)).await? {
        return Err(AppError::NotFound);
    }

    db::events::insert_event(
        &mut tx,
        id,
        EventKind::LateReport,
        EventSource::Server,
        payload,
    )
    .await?;

    tx.commit().await?;

    Err(AppError::Conflict)
}

/// Get a run with its steps.
#[utoipa::path(
    get,
    path = "/runs/{id}",
    operation_id = "get_run",
    tag = "runs",
    params(
        ("id" = Uuid, Path, description = "Run id"),
    ),
    responses(
        (status = OK, description = "Run found", body = RunDetail),
        (status = NOT_FOUND, description = "No such run", body = FlareErrorResponse),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn get_run(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RunDetail>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let Some(res) = db::runs::inspect_run(&mut conn, id).await? else {
        return Err(AppError::NotFound);
    };
    Ok(Json(res))
}

/// List the event timeline of a run.
#[utoipa::path(
    get,
    path = "/runs/{id}/events",
    operation_id = "list_run_events",
    tag = "runs",
    params(
        ("id" = Uuid, Path, description = "Run id"),
    ),
    responses(
        (status = OK, description = "Events of the run", body = Vec<Event>),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such run", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn get_events(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Event>>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let events = select_events(&mut conn, id).await?;
    if events.is_empty() && !db::runs::run_exists(&mut conn, id, None).await? {
        return Err(AppError::NotFound);
    }
    Ok(Json(events))
}

/// List runs, newest first.
#[utoipa::path(
    get,
    path = "/runs",
    operation_id = "list_runs",
    tag = "runs",
    params(ListRuns),
    responses(
        (status = OK, description = "Runs matching the filters", body = Vec<Run>),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn get_runs(
    State(state): State<AppState>,
    Query(q): Query<ListRuns>,
) -> Result<Json<Vec<Run>>, AppError> {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let mut conn = state.pool.acquire().await?;
    Ok(Json(db::runs::list_runs(&mut conn, &q, limit).await?))
}
