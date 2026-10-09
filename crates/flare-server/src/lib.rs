mod app;

pub async fn run(addr: std::net::SocketAddr, db_url: &str) -> anyhow::Result<()> {
    let state = app::AppState::connect(db_url).await?;
    app::serve(state, addr).await
}
