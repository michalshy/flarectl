use crate::app::auth;
use crate::app::error::AppError;
use axum::routing::{get, post, put};
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::job::{CreateJob, Job, RenameJob};
use flare_types::worker::{RegisterWorker, Worker};
use sqlx::query_as;
use uuid::Uuid;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/jobs")
}

async fn post_jobs(
    State(state): State<AppState>,
    Json(job): Json<CreateJob>,
) -> Result<Json<Job>, AppError> {
    let id = Uuid::new_v4();
    let token = auth::generate_token();
    let inserted = query_as!(
        Job,
        "INSERT INTO jobs (id, key, name, token, command, required_labels) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, key, name, command, required_labels, created_at",
        id,
        job.key,
        job.name,
        token,
        job.command,
        job.required_labels
    ).
    fetch_one(&state.pool)
    .await?;
    Ok(Json(inserted))
}

async fn get_jobs(State(state): State<AppState>) -> Result<Json<Vec<Job>>, AppError> {
    todo!()
}

async fn get_job(State(state): State<AppState>, Path(id): Path<Uuid>) {
    todo!()
}

async fn patch_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(job): Json<RenameJob>,
) {
    todo!()
}

async fn delete_job(State(state): State<AppState>, Path(id): Path<Uuid>) {
    todo!()
}

async fn get_job_token(State(state): State<AppState>, Path(id): Path<Uuid>) {
    todo!()
}
