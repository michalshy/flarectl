use crate::app::auth;
use crate::app::error::AppError;
use axum::routing::{get, post};
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::job::{CreateJob, CreatedJob, Job, RenameJob, Secret};
use sqlx::query_as;
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
    Json(job): Json<CreateJob>,
) -> Result<(StatusCode, Json<CreatedJob>), AppError> {
    if !is_valid_key(&job.key) {
        return Err(AppError::Validation(
            "key must be 1-64 chars of a-z, 0-9, -",
        ));
    }

    let id = Uuid::new_v4();
    let token = auth::generate_token();
    let created = query_as!(
        CreatedJob,
        "INSERT INTO jobs (id, key, name, token, command, required_labels) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, token",
        id,
        job.key,
        job.name,
        token,
        job.command,
        job.required_labels
    ).
    fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn get_jobs(State(state): State<AppState>) -> Result<Json<Vec<Job>>, AppError> {
    let selected = query_as!(
        Job,
        "SELECT id, key, name, command, required_labels, created_at FROM jobs ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(selected))
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, AppError> {
    let selected = query_as!(
        Job,
        "SELECT id, key, name, command, required_labels, created_at FROM jobs WHERE id = $1",
        id
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(selected))
}

async fn patch_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(job): Json<RenameJob>,
) -> Result<Json<Job>, AppError> {
    let patched = query_as!(
        Job,
        "UPDATE jobs SET name = $1 WHERE id = $2 RETURNING id, key, name, command, required_labels, created_at",
        job.name,
        id,
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(patched))
}

async fn delete_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!("DELETE from jobs WHERE id = $1", id,)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn get_job_token(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Secret>, AppError> {
    let selected = query_as!(Secret, "SELECT token FROM jobs WHERE id = $1", id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(selected))
}

fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
