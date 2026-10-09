FROM rust:1-bookworm AS builder
WORKDIR /app

# Can be dropped once everything uses rustls.
RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*

COPY . .

ENV SQLX_OFFLINE=true

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin flare \
 && cp target/release/flare /usr/local/bin/flare

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 flare
COPY --from=builder /usr/local/bin/flare /usr/local/bin/flare
USER flare

EXPOSE 8080
CMD ["flare", "server"]
