use std::time::Duration;

use flare_types::run::{
    FinishRun, ListRuns, Run, RunDetail, RunFailure, RunStatus, RunTrigger, StartRun,
};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::app::db::steps::select_steps;

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

/// `job_id = None` skips job scoping — admin routes only.
pub async fn run_exists(
    conn: &mut PgConnection,
    run_id: Uuid,
    job_id: Option<Uuid>,
) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        "SELECT 1 FROM runs WHERE id = $1 AND ($2::uuid IS NULL OR job_id = $2)",
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

pub async fn select_run(conn: &mut PgConnection, run_id: Uuid) -> sqlx::Result<Option<Run>> {
    let run = sqlx::query_as!(
        Run,
        r#"SELECT id, job_id, worker_id,
        trigger        AS "trigger: RunTrigger",
        status         AS "status: RunStatus",
        failure_reason AS "failure_reason: RunFailure",
        started_at, last_heartbeat_at, finished_at, exit_code
        FROM runs WHERE id = $1"#,
        run_id
    )
    .fetch_optional(conn)
    .await?;

    Ok(run)
}

pub async fn inspect_run(conn: &mut PgConnection, run_id: Uuid) -> sqlx::Result<Option<RunDetail>> {
    let Some(run) = select_run(conn, run_id).await? else {
        return Ok(None);
    };

    let steps = select_steps(conn, run_id).await?;

    Ok(Some(RunDetail { run, steps }))
}

pub async fn list_runs(
    conn: &mut PgConnection,
    q: &ListRuns,
    limit: i64,
) -> sqlx::Result<Vec<Run>> {
    sqlx::query_as!(
        Run,
        r#"SELECT id, job_id, worker_id,
                    trigger        AS "trigger: RunTrigger",
                    status         AS "status: RunStatus",
                    failure_reason AS "failure_reason: RunFailure",
                    started_at, last_heartbeat_at, finished_at, exit_code
             FROM runs
             WHERE ($1::uuid        IS NULL OR job_id     = $1)
               AND ($2::text        IS NULL OR status     = $2)
               AND ($3::timestamptz IS NULL OR started_at < $3)
             ORDER BY started_at DESC
             LIMIT $4"#,
        q.job_id,
        q.status as _,
        q.before,
        limit,
    )
    .fetch_all(conn)
    .await
}
