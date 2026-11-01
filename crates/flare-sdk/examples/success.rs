use std::env;

use flare::{job, step};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flare::init(env::var("FLARE_URL")?, env::var("FLARE_TOKEN")?);
    nightly_build().await
}

#[job]
async fn nightly_build() -> anyhow::Result<()> {
    let version = fetch().await?;
    build(&version).await
}

#[step]
async fn fetch() -> anyhow::Result<String> {
    Ok("1.4.2".to_string())
}

#[step(name = "compile sources")]
async fn build(version: &str) -> anyhow::Result<()> {
    println!("building {version}");
    Ok(())
}
