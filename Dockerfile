# Stage 1: Build frontend
FROM oven/bun:1 AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/bun.lock ./
RUN bun install --frozen-lockfile
COPY frontend/ .
RUN bun run build

# Stage 2: Build server
FROM rust:1-bookworm AS builder
WORKDIR /app

# Cache dependency builds: copy manifests first
COPY Cargo.toml Cargo.lock ./
COPY crates/req1-server/Cargo.toml crates/req1-server/Cargo.toml
COPY crates/req1-core/Cargo.toml crates/req1-core/Cargo.toml
COPY crates/req1-reqif/Cargo.toml crates/req1-reqif/Cargo.toml
COPY crates/req1-cli/Cargo.toml crates/req1-cli/Cargo.toml
COPY entity/Cargo.toml entity/Cargo.toml
COPY migration/Cargo.toml migration/Cargo.toml

# Create dummy source files for dependency caching
RUN mkdir -p crates/req1-server/src crates/req1-core/src crates/req1-reqif/src crates/req1-cli/src entity/src migration/src && \
    echo "fn main() {}" > crates/req1-server/src/main.rs && \
    echo "" > crates/req1-server/src/lib.rs && \
    echo "" > crates/req1-core/src/lib.rs && \
    echo "" > crates/req1-reqif/src/lib.rs && \
    echo "fn main() {}" > crates/req1-cli/src/main.rs && \
    echo "" > entity/src/lib.rs && \
    echo "" > migration/src/lib.rs
RUN cargo build --release --bin req1-server 2>/dev/null || true

# Copy real source and build
COPY crates/ crates/
COPY entity/ entity/
COPY migration/ migration/
RUN cargo build --release --bin req1-server

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 curl && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -r -u 1000 -m appuser
WORKDIR /app

COPY --from=builder /app/target/release/req1-server /app/req1-server
COPY --from=frontend /app/frontend/dist /app/static

ENV STATIC_DIR=/app/static
EXPOSE 8080

USER appuser
ENTRYPOINT ["/app/req1-server"]
