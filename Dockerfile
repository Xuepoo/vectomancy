# syntax=docker/dockerfile:1
FROM rust:slim-bookworm AS builder
WORKDIR /usr/src/app

# Install build dependencies for wgpu/Vulkan and FFmpeg
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libvulkan-dev \
    clang \
    libavcodec-dev \
    libavformat-dev \
    libavutil-dev \
    libswscale-dev \
    libswresample-dev \
    libavdevice-dev \
    libavfilter-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests and workspaces
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY cli ./cli
COPY text ./text
COPY video ./video
COPY benches ./benches
COPY templates ./templates

# Build the release binary for the CLI subcommand
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release --bin vectomancy && \
    cp ./target/release/vectomancy /tmp/vectomancy

# Runtime Stage
FROM debian:bookworm-slim

# Install runtime dependencies for wgpu (Vulkan) and FFmpeg libs
RUN apt-get update && apt-get upgrade -y && apt-get install -y --no-install-recommends \
    libvulkan1 \
    mesa-vulkan-drivers \
    vulkan-tools \
    libavcodec-dev \
    libavformat-dev \
    libavutil-dev \
    libswscale-dev \
    libswresample-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /tmp/vectomancy /usr/local/bin/vectomancy

# Set default working directory for external data mounts
WORKDIR /data

ENTRYPOINT ["vectomancy"]
