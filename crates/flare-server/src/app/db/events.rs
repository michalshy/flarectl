use flare_types::event::{Event, EventKind, EventSource};
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

pub async fn select_events(conn: &mut PgConnection, run_id: Uuid) -> sqlx::Result<Vec<Event>> {
    let events = sqlx::query_as!(
        Event,
        r#"SELECT id, run_id,
            kind   AS "kind: EventKind",
            source AS "source: EventSource",
            payload, created_at
            FROM events
            WHERE run_id = $1
            ORDER BY id"#,
        run_id
    )
    .fetch_all(conn)
    .await?;

    Ok(events)
}
