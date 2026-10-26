use std::time::Duration;

mod app;

pub async fn run(
    addr: std::net::SocketAddr,
    db_url: &str,
    heartbeat_timeout: Duration,
) -> anyhow::Result<()> {
    let state = app::AppState::connect(db_url).await?;
    app::serve(state, addr, heartbeat_timeout).await
}
