FROM rust:1.75-slim-bookworm AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/vectomancy /usr/local/bin/vectomancy
ENTRYPOINT ["vectomancy"]