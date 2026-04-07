FROM rust:1.89-bookworm AS builder
WORKDIR /app

COPY . .
RUN cargo build --release -p api-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /app/target/release/api-gateway /usr/local/bin/api-gateway

CMD ["api-gateway"]
