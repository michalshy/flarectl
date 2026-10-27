use flare_types::event::{EventKind, EventSource};
use sqlx::PgConnection;
use uuid::Uuid;

pub async fn insert_event(
    conn: &mut PgConnection,
    run_id: Uuid,
    kind: EventKind,
    source: EventSource,
    payload: serde_json::Value,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO events (run_id, kind, source, payload) VALUES ($1, $2, $3, $4)",
        run_id,
        kind as _,
        source as _,
        payload
    )
    .execute(conn)
    .await?;
    Ok(())
}
