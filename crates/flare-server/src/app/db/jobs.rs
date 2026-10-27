use flare_types::job::{CreateJob, CreatedJob, Job, RenameJob, Secret};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::app::auth;

pub async fn create_job(conn: &mut PgConnection, input: CreateJob) -> sqlx::Result<CreatedJob> {
    let id = Uuid::new_v4();
    let token = auth::generate_token();
    let job = sqlx::query_as!(
        CreatedJob,
        "INSERT INTO jobs (id, key, name, token, command, required_labels) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, token",
        id,
        input.key,
        input.name,
        token,
        input.command,
        input.required_labels
    ).
    fetch_one(conn)
    .await?;
    Ok(job)
}

pub async fn get_jobs(conn: &mut PgConnection) -> sqlx::Result<Vec<Job>> {
    let jobs = sqlx::query_as!(
        Job,
        "SELECT id, key, name, command, required_labels, created_at FROM jobs ORDER BY created_at DESC",
    )
    .fetch_all(conn)
    .await?;

    Ok(jobs)
}

pub async fn get_job(conn: &mut PgConnection, job_id: Uuid) -> sqlx::Result<Job> {
    let job = sqlx::query_as!(
        Job,
        "SELECT id, key, name, command, required_labels, created_at FROM jobs WHERE id = $1",
        job_id
    )
    .fetch_one(conn)
    .await?;

    Ok(job)
}

pub async fn rename_job(
    conn: &mut PgConnection,
    job_id: Uuid,
    job: RenameJob,
) -> sqlx::Result<Job> {
    let job = sqlx::query_as!(
        Job,
        "UPDATE jobs SET name = $1 WHERE id = $2 RETURNING id, key, name, command, required_labels, created_at",
        job.name,
        job_id,
    )
    .fetch_one(conn)
    .await?;
    Ok(job)
}

pub async fn delete_job(conn: &mut PgConnection, job_id: Uuid) -> sqlx::Result<u64> {
    let result = sqlx::query!("DELETE from jobs WHERE id = $1", job_id)
        .execute(conn)
        .await?;

    Ok(result.rows_affected())
}

pub async fn get_job_token(conn: &mut PgConnection, job_id: Uuid) -> sqlx::Result<Secret> {
    let secret = sqlx::query_as!(Secret, "SELECT token FROM jobs WHERE id = $1", job_id)
        .fetch_one(conn)
        .await?;

    Ok(secret)
}
