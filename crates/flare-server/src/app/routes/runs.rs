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
use uuid::Uuid;

use crate::app::{AppState, db};

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

    let new_run = db::runs::put_run(&mut tx, id, job.id, input).await?;
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

async fn post_heartbeat(
    State(state): State<AppState>,
    job: AuthJob,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.pool.acquire().await?;
    let rows = db::runs::post_heartbeat(&mut conn, id, job.id).await?;

    if rows == 0 {
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

    let finished = db::runs::post_finish(&mut tx, id, job.id, input, reason).await?;

    if finished.is_some() {
        db::events::insert_event(
            &mut tx,
            id,
            EventKind::Finished,
            EventSource::Server,
            payload,
        )
        .await?;

        db::steps::fail_running_steps(&mut tx, finished.unwrap()).await?;

        tx.commit().await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    if !db::runs::run_exists(&mut tx, id, job.id).await? {
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
