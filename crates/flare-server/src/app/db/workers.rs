use flare_types::worker::{RegisterWorker, Worker};
use sqlx::PgConnection;
use uuid::Uuid;

pub async fn get_workers(conn: &mut PgConnection) -> sqlx::Result<Vec<Worker>> {
    let workers = sqlx::query_as!(
        Worker,
        "SELECT id, name, version, labels, registered_at, last_seen_at FROM workers"
    )
    .fetch_all(conn)
    .await?;
    Ok(workers)
}

pub async fn get_worker(conn: &mut PgConnection, worker_id: Uuid) -> sqlx::Result<Worker> {
    let worker = sqlx::query_as!(
        Worker,
        "SELECT id, name, version, labels, registered_at, last_seen_at FROM workers WHERE id = $1",
        worker_id
    )
    .fetch_one(conn)
    .await?;
    Ok(worker)
}

pub async fn insert_worker(
    conn: &mut PgConnection,
    worker_id: Uuid,
    input: RegisterWorker,
) -> sqlx::Result<Worker> {
    let inserted = sqlx::query_as!(
        Worker,
        "
            INSERT INTO workers (id, name, version)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                version = EXCLUDED.version,
                last_seen_at = now()
            RETURNING id, name, version, labels, registered_at, last_seen_at
        ",
        worker_id,
        input.name,
        input.version,
    )
    .fetch_one(conn)
    .await?;
    Ok(inserted)
}

pub async fn touch_heartbeat(conn: &mut PgConnection, worker_id: Uuid) -> sqlx::Result<u64> {
    let request = sqlx::query!(
        "UPDATE workers SET last_seen_at = now() WHERE id = $1",
        worker_id
    )
    .execute(conn)
    .await?;
    Ok(request.rows_affected())
}
