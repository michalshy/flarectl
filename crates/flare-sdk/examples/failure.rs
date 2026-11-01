use std::env;

use flare::{job, step};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flare::init(env::var("FLARE_URL")?, env::var("FLARE_TOKEN")?);
    nightly_build().await
}

#[job]
async fn nightly_build() -> anyhow::Result<()> {
    fetch().await?;
    upload().await
}

#[step]
async fn fetch() -> anyhow::Result<()> {
    println!("hi");
    Ok(())
}

// fails the step, then the job
#[step]
async fn upload() -> anyhow::Result<()> {
    anyhow::bail!("upload failed")
}
