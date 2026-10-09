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
        Command::Server { listen, db_url } => flare_server::run(listen, &db_url).await,
        Command::Worker { url } => flare_worker::run(url).await,
    }
}
