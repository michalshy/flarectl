CREATE TABLE jobs (
    id              uuid PRIMARY KEY,
    key             text NOT NULL UNIQUE,
    name            text NOT NULL,
    command         text,
    required_labels jsonb NOT NULL DEFAULT '{}',
    token_hash      text NOT NULL UNIQUE,
    created_at      timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE runs (
    id                uuid PRIMARY KEY,
    job_id            uuid NOT NULL REFERENCES jobs ON DELETE CASCADE,
    worker_id         uuid REFERENCES workers ON DELETE SET NULL,
    trigger           text NOT NULL CHECK (trigger IN ('external','schedule','ui','missed')),
    status            text NOT NULL CHECK (status IN ('running','succeeded','failed','aborted')),
    failure_reason    text CHECK (failure_reason IN ('reported','heartbeat_timeout','missed')),
    started_at        timestamptz NOT NULL DEFAULT now(),
    last_heartbeat_at timestamptz NOT NULL DEFAULT now(),
    finished_at       timestamptz,
    exit_code         int,
    CHECK ((status = 'failed') = (failure_reason IS NOT NULL)),
    CHECK ((status = 'running') = (finished_at IS NULL))
);
CREATE INDEX runs_job_started_idx ON runs (job_id, started_at DESC);
CREATE INDEX runs_running_hb_idx  ON runs (last_heartbeat_at) WHERE status = 'running';

CREATE TABLE steps (
    id          uuid PRIMARY KEY,
    run_id      uuid NOT NULL REFERENCES runs ON DELETE CASCADE,
    name        text NOT NULL,
    position    int  NOT NULL,
    status      text NOT NULL CHECK (status IN ('running','succeeded','failed')),
    started_at  timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz,
    UNIQUE (run_id, position),
    CHECK ((status = 'running') = (finished_at IS NULL))
);

CREATE TABLE run_events (
    id         bigserial PRIMARY KEY,
    run_id     uuid NOT NULL REFERENCES runs ON DELETE CASCADE,
    kind       text NOT NULL,
    payload    jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX run_events_run_idx ON run_events (run_id, id);
