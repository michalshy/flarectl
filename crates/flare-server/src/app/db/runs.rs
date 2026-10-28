use std::time::Duration;

use flare_types::run::{FinishRun, RunFailure, RunStatus, StartRun};
use sqlx::PgConnection;
use uuid::Uuid;

pub async fn start_run(
    conn: &mut PgConnection,
    run_id: Uuid,
    job_id: Uuid,
    input: StartRun,
) -> sqlx::Result<Option<Uuid>> {
    let id = sqlx::query_scalar!(
        "INSERT INTO runs (id, job_id, worker_id, trigger, status)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE SET id = runs.id
        WHERE runs.job_id = EXCLUDED.job_id AND runs.status = $5
        RETURNING id",
        run_id,
        job_id,
        input.worker_id,
        input.trigger as _,
        RunStatus::Running as _
    )
    .fetch_optional(conn)
    .await?;

    Ok(id)
}

pub async fn touch_heartbeat(
    conn: &mut PgConnection,
    run_id: Uuid,
    job_id: Uuid,
) -> sqlx::Result<u64> {
    let request = sqlx::query!(
        "UPDATE runs SET last_heartbeat_at = now() WHERE id = $1 AND job_id = $2 AND status = $3",
        run_id,
        job_id,
        RunStatus::Running as _
    )
    .execute(conn)
    .await?;
    Ok(request.rows_affected())
}

pub async fn finish_run(
    conn: &mut PgConnection,
    run_id: Uuid,
    job_id: Uuid,
    input: FinishRun,
    reason: Option<RunFailure>,
) -> sqlx::Result<Option<Uuid>> {
    let updated = sqlx::query_scalar!(
        "UPDATE runs
        SET status = $4, failure_reason = $5, exit_code = $6, finished_at = now()
        WHERE id = $1 AND job_id = $2 AND status = $3
        RETURNING id",
        run_id,
        job_id,
        RunStatus::Running as _,
        input.final_status as _,
        reason as _,
        input.exit_code
    )
    .fetch_optional(conn)
    .await?;
    Ok(updated)
}

pub async fn run_exists(conn: &mut PgConnection, run_id: Uuid, job_id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        "SELECT 1 FROM runs WHERE id = $1 AND job_id = $2",
        run_id,
        job_id
    )
    .fetch_optional(conn)
    .await
    .map(|row| row.is_some())
}

pub async fn finish_stale(conn: &mut PgConnection, timeout: Duration) -> sqlx::Result<Vec<Uuid>> {
    let ids = sqlx::query_scalar!(
        "UPDATE runs
        SET status = $1, failure_reason = $2, finished_at = now()
        WHERE status = $3 AND last_heartbeat_at < now() - $4::interval
        RETURNING id",
        RunStatus::Failed as _,
        RunFailure::HeartbeatTimeout as _,
        RunStatus::Running as _,
        timeout as _
    )
    .fetch_all(conn)
    .await?;

    Ok(ids)
}
