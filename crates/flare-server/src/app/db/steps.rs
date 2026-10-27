use flare_types::{
    run::RunStatus,
    step::{CreateStep, FinishStep, StepStatus},
};
use sqlx::PgConnection;
use uuid::Uuid;

pub async fn fail_running_steps(conn: &mut PgConnection, run_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE steps SET status = $1, finished_at = now()
        WHERE run_id = $2 AND status = $3",
        StepStatus::Failed as _,
        run_id,
        StepStatus::Running as _
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn start_step(
    conn: &mut PgConnection,
    step_id: Uuid,
    run_id: Uuid,
    job_id: Uuid,
    input: CreateStep,
) -> sqlx::Result<Option<Uuid>> {
    let started = sqlx::query_scalar!(
        "INSERT INTO steps (id, run_id, name, position, status)
        SELECT $1, $2, $3, $4, $5
        WHERE EXISTS (SELECT 1 FROM runs WHERE id = $2 AND job_id = $6 AND status = $7)
        ON CONFLICT (id) DO UPDATE SET id = steps.id WHERE steps.run_id = EXCLUDED.run_id
        RETURNING id",
        step_id,
        run_id,
        input.name,
        input.position,
        StepStatus::Running as _,
        job_id,
        RunStatus::Running as _,
    )
    .fetch_optional(conn)
    .await?;
    Ok(started)
}

pub async fn finish_step(
    conn: &mut PgConnection,
    step_id: Uuid,
    run_id: Uuid,
    job_id: Uuid,
    input: FinishStep,
) -> sqlx::Result<Option<Uuid>> {
    let finished = sqlx::query_scalar!(
        "UPDATE steps SET status = $1, finished_at = now()
        FROM runs
        WHERE steps.id = $2 AND steps.run_id = $3 AND steps.status = $4
          AND runs.id = steps.run_id AND runs.job_id = $5
        RETURNING steps.id",
        input.status as _,
        step_id,
        run_id,
        StepStatus::Running as _,
        job_id,
    )
    .fetch_optional(conn)
    .await?;
    Ok(finished)
}

pub async fn step_exists(
    conn: &mut PgConnection,
    step_id: Uuid,
    run_id: Uuid,
    job_id: Uuid,
) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        "SELECT 1 FROM steps
        JOIN runs ON runs.id = steps.run_id
        WHERE steps.id = $1 AND steps.run_id = $2 AND runs.job_id = $3",
        step_id,
        run_id,
        job_id
    )
    .fetch_optional(conn)
    .await
    .map(|row| row.is_some())
}
