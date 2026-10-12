# CLAUDE.md — Flare

## How to work with flare

- **Do not write or edit code unless I explicitly ask.** I'm building Flare myself to learn.
- Default mode is advice: explain trade-offs, point out bugs, suggest the next step, review what I paste.
- Provide code as examples, not production-ready.
- Don't run commands that change the repo, the database or containers without asking. Reading files,
  `cargo check`, `cargo test` and `docker compose ps/logs` are fine.
- If I ask "why", explain the concept before giving the fix.
- Keep answers short and concrete. Point to the exact file/line when reviewing.

## What Flare is

A lightweight, self-hosted alternative to Temporal/Jenkins for running and observing jobs.
Any process, started anywhere (cron, CI, laptop, container), **reports its own execution** to the
Flare server over a simple HTTP API. The server records and displays runs, detects dead processes
via heartbeats, and alerts when an expected run didn't happen. Workers exist only to run scheduled
jobs and jobs triggered from the UI.

## Stack

- **Server:** Rust, axum, tokio, sqlx 0.9 with raw SQL (no ORM), PostgreSQL only.
- **Worker:** Rust, same binary as the server.
- **CLI:** one binary `flare` (`crates/flare-cli`) with subcommands `server`, `worker`, later `exec`.
  clap with `derive` + `env`. Tokio runtime and tracing init live only in the CLI.
- **UI:** TypeScript, React + Vite, API client generated from OpenAPI (not started yet).
- **SDKs (later):** Rust, TypeScript, Python, Go.

## Repo layout

```
flarectl/
  Cargo.toml                 # workspace root, shared versions in [workspace.dependencies]
  Makefile                   # dev commands
  Dockerfile                 # builds `flare`, BuildKit cache mounts, SQLX_OFFLINE=true
  docker-compose.yml         # postgres + server
  crates/
    flare-cli/               # bin `flare`, clap, #[tokio::main]
    flare-server/            # lib: AppState, router, handlers, db
      migrations/            # sqlx migrations, embedded via sqlx::migrate!()
    flare-worker/            # lib: register + heartbeat loop
    flare-sdk/               # lib (stub)
```

Libraries expose `pub async fn run(...) -> anyhow::Result<()>`; they never create a runtime or
read env vars themselves. Config flows CLI → `run()`.
