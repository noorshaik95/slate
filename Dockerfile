# Build Stage
FROM rust:1.90 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/axum-grafana-example /usr/local/bin/app
EXPOSE 8080 8080
CMD ["app"]
