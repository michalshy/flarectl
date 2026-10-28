use std::time::Duration;

use flare_types::event::{EventKind, EventSource};
use tracing::{error, warn};

use crate::app::{AppState, db};

pub async fn sweeper(state: AppState, timeout: Duration) {
    let mut ticker = tokio::time::interval((timeout + Duration::from_secs(3)) / 3);
    loop {
        ticker.tick().await;
        if let Err(error) = sweep_once(&state, timeout).await {
            error!("Sweep failed: {}", error);
        }
    }
}

async fn sweep_once(state: &AppState, timeout: Duration) -> sqlx::Result<()> {
    let mut tx = state.pool.begin().await?;

    let ids = db::runs::finish_stale(&mut tx, timeout).await?;

    for id in ids.clone() {
        db::steps::fail_running_steps(&mut tx, id).await?;
        db::events::insert_event(
            &mut tx,
            id,
            EventKind::HeartbeatTimeout,
            EventSource::Server,
            serde_json::json!({}),
        )
        .await?;
    }

    tx.commit().await?;

    for id in ids {
        warn!("Run finished due to exceeding heartbeat: {}", id)
    }

    Ok(())
}
