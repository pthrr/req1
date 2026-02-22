FROM ubuntu:24.04

# System deps
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       curl ca-certificates git sudo procps psmisc \
       build-essential pkg-config libssl-dev \
       docker.io docker-compose-v2 \
       postgresql-client \
       unzip \
    && rm -rf /var/lib/apt/lists/*

# Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
    && . /root/.cargo/env \
    && rustup component add rustfmt clippy rust-analyzer rust-src

# Bun
RUN curl -fsSL https://bun.sh/install | bash

# go-task
RUN sh -c "$(curl --location https://taskfile.dev/install.sh)" -- -d -b /usr/local/bin

ENV PATH="/root/.cargo/bin:/root/.bun/bin:${PATH}"
ENV RUST_SRC_PATH="/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library"
