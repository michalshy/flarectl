# Flare

A lightweight, self-hosted platform for running and observing jobs written as plain code.

> **Status: early development.** The server works, but the SDK, the UI and most of what's described
> below are still being built.

## Why

Flare sits between two kinds of tools:

- **Jenkins** defines pipelines as config, with weak observability and retries.
- **Temporal** gives you durable execution, but your code has to follow its determinism rules.

Many jobs need neither. A nightly build, a backup or a data import is usually just a program. You
want to know whether it ran, which step failed and why, and to retry it without starting over.

## The idea

- **Code first.** A job is an ordinary program. There's no pipeline DSL.
- **Report from anywhere.** Jobs report to the server over a simple HTTP API, whether they run from
  cron, CI, a laptop, a container or a build machine.
- **Step-level retries, not durable execution.** A failed run can be retried from the failed step.
  Steps that already succeeded are skipped and their recorded output is reused. There's no replay,
  and no determinism rules for your code.
- **Detect what went wrong.** Heartbeats catch dead processes, schedules catch runs that should
  have happened but didn't, and alerts tell you about it.
- **Workers are optional.** They only start scheduled jobs and jobs triggered from the UI.

## What a job will look like

The planned Rust SDK:

```rust
use flare::{job, step};

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
```

Each run, its steps and their timings show up on the server, with no pipeline definition anywhere.

## How it fits together

- **Server:** records runs, steps and an event timeline. It never runs your code.
- **SDK:** used inside your program to report to the server. Rust comes first, other languages later.
- **Worker:** an optional trigger that starts a process, which then reports for itself.
- **UI:** a dashboard for runs, steps and timelines.

## Stack

Rust, axum and PostgreSQL on the server, with React and TypeScript for the UI.
