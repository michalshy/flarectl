use crate::app::db;
use crate::app::error::AppError;
use axum::routing::{get, post};
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::job::{CreateJob, CreatedJob, Job, RenameJob, Secret};
use uuid::Uuid;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs", post(post_jobs).get(get_jobs))
        .route(
            "/jobs/{id}",
            get(get_job).patch(patch_job).delete(delete_job),
        )
        .route("/jobs/{id}/token", get(get_job_token))
}

async fn post_jobs(
    State(state): State<AppState>,
    Json(input): Json<CreateJob>,
) -> Result<(StatusCode, Json<CreatedJob>), AppError> {
    if !is_valid_key(&input.key) {
        return Err(AppError::Validation(
            "key must be 1-64 chars of a-z, 0-9, -",
        ));
    }
    let mut conn = state.pool.acquire().await?;
    let created = db::jobs::create_job(&mut conn, input).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn get_jobs(State(state): State<AppState>) -> Result<Json<Vec<Job>>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let jobs = db::jobs::get_jobs(&mut conn).await?;
    Ok(Json(jobs))
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let job = db::jobs::get_job(&mut conn, id).await?;
    Ok(Json(job))
}

async fn patch_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(job): Json<RenameJob>,
) -> Result<Json<Job>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let job = db::jobs::rename_job(&mut conn, id, job).await?;
    Ok(Json(job))
}

async fn delete_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.pool.acquire().await?;
    let rows = db::jobs::delete_job(&mut conn, id).await?;
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn get_job_token(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Secret>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let secret = db::jobs::get_job_token(&mut conn, id).await?;
    Ok(Json(secret))
}

fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
