use std::{env, time::Duration};

use flare::{job, step};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flare::init(env::var("FLARE_URL")?, env::var("FLARE_TOKEN")?);
    nightly_build().await
}

#[job]
async fn nightly_build() -> anyhow::Result<()> {
    // default outlives the server's 30s heartbeat timeout
    let secs = env::var("SLEEP_SECS").map_or(Ok(40), |s| s.parse())?;
    cook(secs).await
}

#[step]
async fn cook(secs: u64) -> anyhow::Result<()> {
    println!("start");
    tokio::time::sleep(Duration::from_secs(secs)).await;
    println!("done");
    Ok(())
}
