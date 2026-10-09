-- Add up migration script here
CREATE TABLE workers (
    id              uuid PRIMARY KEY,
    name            text NOT NULL,
    version         text,
    labels          jsonb NOT NULL DEFAULT '{}',
    registered_at   timestamptz NOT NULL DEFAULT now(),
    last_seen_at    timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX workers_last_seen_at_idx ON workers (last_seen_at DESC);
