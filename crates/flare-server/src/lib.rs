use axum::{Router, routing::get};

pub async fn run(addr: std::net::SocketAddr) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/workers", get(workers));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "Ok."
}

async fn workers() -> &'static str {
    "Here will be many many workers."
}
