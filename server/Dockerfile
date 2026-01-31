# Build stage: compile the Rust server
FROM rust:1-bookworm AS builder

WORKDIR /app

# Copy engine and server (server depends on engine via path)
COPY engine /app/engine
COPY server /app/server

RUN cargo build --release --manifest-path /app/server/Cargo.toml

# Run stage: minimal image with Stockfish
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends stockfish ca-certificates \
    && ln -sf /usr/games/stockfish /usr/local/bin/stockfish \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/server/target/release/server /app/server

# Render sets PORT (default 10000)
ENV PORT=10000
EXPOSE 10000

CMD ["/app/server"]
