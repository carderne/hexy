FROM rust:1.90.0-slim-trixie AS builder
WORKDIR /app
RUN apt-get update \
  && apt-get install -y pkg-config libssl-dev libsqlite3-dev
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
WORKDIR /app
RUN apt-get update \
  && apt-get install -y libssl3 ca-certificates libsqlite3-0 curl \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/hexy /app/app
COPY --from=builder /app/static /app/static
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/diesel.toml /app/diesel.toml
COPY --from=builder /app/Rocket.toml /app/Rocket.toml

EXPOSE 8000

CMD ["/app/app"]
