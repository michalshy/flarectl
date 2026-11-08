use crate::app::error::AppError;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use flare_types::error::FlareErrorResponse;
use flare_types::worker::{RegisterWorker, Worker};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::app::{AppState, db};

pub fn admin_api() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_workers))
        .routes(routes!(put_worker, get_worker))
        .routes(routes!(post_heartbeat))
}

/// List workers.
#[utoipa::path(
    get,
    path = "/workers",
    operation_id = "list_workers",
    tag = "workers",
    responses(
        (status = OK, description = "Workers", body = Vec<Worker>),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn get_workers(State(state): State<AppState>) -> Result<Json<Vec<Worker>>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let workers = db::workers::get_workers(&mut conn).await?;
    Ok(Json(workers))
}

/// Get a worker.
#[utoipa::path(
    get,
    path = "/workers/{id}",
    operation_id = "get_worker",
    tag = "workers",
    params(
        ("id" = Uuid, Path, description = "Worker id"),
    ),
    responses(
        (status = OK, description = "Worker found", body = Worker),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such worker", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn get_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Worker>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let worker = db::workers::get_worker(&mut conn, id).await?;
    Ok(Json(worker))
}

/// Register a worker, or update it if the id exists.
#[utoipa::path(
    put,
    path = "/workers/{id}",
    operation_id = "register_worker",
    tag = "workers",
    params(
        ("id" = Uuid, Path, description = "Worker id"),
    ),
    request_body = RegisterWorker,
    responses(
        (status = OK, description = "Worker registered", body = Worker),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn put_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<RegisterWorker>,
) -> Result<Json<Worker>, AppError> {
    let mut conn = state.pool.acquire().await?;
    let inserted = db::workers::insert_worker(&mut conn, id, input).await?;
    Ok(Json(inserted))
}

/// Record a worker heartbeat.
#[utoipa::path(
    post,
    path = "/workers/{id}/heartbeat",
    operation_id = "heartbeat_worker",
    tag = "workers",
    params(
        ("id" = Uuid, Path, description = "Worker id"),
    ),
    responses(
        (status = NO_CONTENT, description = "Heartbeat recorded"),
        (status = UNAUTHORIZED, description = "Missing or bad auth token", body = FlareErrorResponse),
        (status = NOT_FOUND, description = "No such worker", body = FlareErrorResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = FlareErrorResponse),
    )
)]
async fn post_heartbeat(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.pool.acquire().await?;
    if db::workers::touch_heartbeat(&mut conn, id).await? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
