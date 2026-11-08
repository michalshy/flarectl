use std::future::Future;

pub use flare_sdk_macros::{job, step};

pub fn init(_url: String, _token: String) {
    todo!("store the client for #[job] and #[step]")
}

#[allow(dead_code)]
pub struct Client {
    url: String,
    token: String,
}

impl Client {
    pub fn new(url: String, token: String) -> Self {
        Self { url, token }
    }

    pub async fn run<F, Fut>(&self, _job: F) -> anyhow::Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = anyhow::Result<()>>,
    {
        todo!("start the run, call the job, finish the run")
    }
}
