use crate::app::db;
use crate::app::error::AppError;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::error::FlareErrorResponse;
use flare_types::job::{CreateJob, CreatedJob, Job, RenameJob, Secret};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::app::AppState;

pub fn admin_api() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(post_jobs, get_jobs))
        .routes(routes!(get_job, patch_job, delete_job))
        .routes(routes!(get_job_token))
}

/// Create a job. The response holds its token.
#[utoipa::path(
    post,
    path = "/jobs",
    operation_id = "create_job",
    tag = "jobs",
    request_body = CreateJob,
    responses(
        (status = CREATED, description = "Job created", body = CreatedJob),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = CONFLICT, description = "Key already taken", body = FlareErrorResponse),
        (status = UNPROCESSABLE_ENTITY, description = "Invalid key", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
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

/// List jobs.
#[utoipa::path(
    get,
    path = "/jobs",
    operation_id = "list_jobs",
    tag = "jobs",
    responses(
        (status = OK, description = "Jobs", body = Vec<Job>),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn get_jobs(State(state): State<AppState>) -> Result<Json<Vec<Job>>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let jobs = db::jobs::get_jobs(&mut conn).await?;
    Ok(Json(jobs))
}

/// Get a job.
#[utoipa::path(
    get,
    path = "/jobs/{id}",
    operation_id = "get_job",
    tag = "jobs",
    params(
        ("id" = Uuid, Path, description = "Job id"),
    ),
    responses(
        (status = OK, description = "Job found", body = Job),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such job", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let job = db::jobs::get_job(&mut conn, id).await?;
    Ok(Json(job))
}

/// Rename a job.
#[utoipa::path(
    patch,
    path = "/jobs/{id}",
    operation_id = "rename_job",
    tag = "jobs",
    params(
        ("id" = Uuid, Path, description = "Job id"),
    ),
    request_body = RenameJob,
    responses(
        (status = OK, description = "Job renamed", body = Job),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such job", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn patch_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(job): Json<RenameJob>,
) -> Result<Json<Job>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let job = db::jobs::rename_job(&mut conn, id, job).await?;
    Ok(Json(job))
}

/// Delete a job with its runs.
#[utoipa::path(
    delete,
    path = "/jobs/{id}",
    operation_id = "delete_job",
    tag = "jobs",
    params(
        ("id" = Uuid, Path, description = "Job id"),
    ),
    responses(
        (status = NO_CONTENT, description = "Job deleted"),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such job", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
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

/// Get the token a job reports with.
#[utoipa::path(
    get,
    path = "/jobs/{id}/token",
    operation_id = "get_job_token",
    tag = "jobs",
    params(
        ("id" = Uuid, Path, description = "Job id"),
    ),
    responses(
        (status = OK, description = "Job token", body = Secret),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such job", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
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
