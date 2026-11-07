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

A lightweight, self-hosted platform for running and observing jobs written as **plain code**,
sitting between Jenkins (pipelines as config, weak observability and retries) and Temporal
(durable execution, deterministic replay).

- **Code-first.** Jobs are ordinary programs. An SDK (or `flare exec` for any command) reports the
  run, its steps and logs to the Flare server over a simple HTTP API. No pipeline DSL.
- **Report from anywhere.** cron, CI (including inside Jenkins), a laptop, a container, a build
  machine. The server records and displays runs, steps and an event timeline.
- **Step-level retries, not durable execution.** A failed run can be retried from the failed
  step: steps that already succeeded are skipped and their recorded output reused. No replay,
  no determinism rules for user code.
- **Detect what went wrong.** Heartbeats detect dead processes; schedules detect runs that should
  have happened but didn't; alerts notify.
- **Workers are optional.** They only run scheduled jobs and jobs triggered from the UI.

## Stack

- **Server:** Rust, axum, tokio, sqlx 0.9 with raw SQL (no ORM), PostgreSQL only.
- **Worker:** Rust, same binary as the server.
- **CLI:** one binary `flare` (`crates/flare-cli`) with subcommands `server`, `worker`, later `exec`.
  clap with `derive` + `env`. Tokio runtime and tracing init live only in the CLI.
- **UI:** TypeScript, React + Vite, API client generated from OpenAPI (not started yet).
- **SDKs:** Rust first (`crates/flare-sdk`); TypeScript, Python, Go later.

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

## sqlx offline cache

`query!` macros are checked against a live DB at compile time; Docker and `make check` build with
`SQLX_OFFLINE=true` and read `.sqlx/` instead. After adding or changing a `query!`, run
`make prepare` and commit `.sqlx/`, or offline builds fail.

## OpenAPI (utoipa)

The spec is committed at `api/openapi.json` and checked by a test. After changing a documented
route or a `ToSchema` type, run `make openapi` and commit the file, or `make check` fails.

- Output types (responses): mark `Option` fields
  `#[cfg_attr(feature = "utoipa", schema(required = true))]`, because serde always sends the key
  (as `null`). Skip this only if the field uses `skip_serializing_if`.
- Input types (request bodies, query params): leave `Option` fields optional.
- Don't share one type between request and response; split it instead.
