# syntax=docker/dockerfile:1
FROM rust:slim-bookworm AS builder
WORKDIR /usr/src/app

# Install build dependencies for wgpu/Vulkan
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libvulkan-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests and cache retrieval to optimize rebuilds
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY benches ./benches
COPY templates ./templates

# Use BuildKit cache mounts to prevent re-downloading and re-compiling crates
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release && \
    cp ./target/release/vectomancy /tmp/vectomancy

# Runtime Stage
FROM debian:bookworm-slim

# Install wgpu runtime dependencies (Vulkan and graphics libraries) and patch CVE vulnerabilities
RUN apt-get update && apt-get upgrade -y && apt-get install -y --no-install-recommends \
    libvulkan1 \
    mesa-vulkan-drivers \
    vulkan-tools \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /tmp/vectomancy /usr/local/bin/vectomancy

# Set default working directory for external data mounts
WORKDIR /data

ENTRYPOINT ["vectomancy"]
