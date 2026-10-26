use std::time::Duration;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "flare")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Server {
        #[arg(long, env = "FLARE_LISTEN", default_value = "0.0.0.0:8080")]
        listen: std::net::SocketAddr,
        #[arg(long, env = "DATABASE_URL")]
        db_url: String,
        #[arg(long, env = "FLARE_HEARTBEAT_TIMEOUT_SECS", default_value_t = 30)]
        heartbeat_timeout: u64,
    },
    Worker {
        #[arg(long, env = "FLARE_URL", default_value = "http://localhost:8080")]
        url: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    match Cli::parse().command {
        Command::Server {
            listen,
            db_url,
            heartbeat_timeout,
        } => flare_server::run(listen, &db_url, Duration::from_secs(heartbeat_timeout)).await,
        Command::Worker { url } => flare_worker::run(url).await,
    }
}
