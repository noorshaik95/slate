# Build Stage
FROM rust:1.90 as builder

# Install protobuf compiler for proto compilation
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy proto files for proto compilation (needed for build.rs)
COPY proto ./proto
COPY services/api-gateway/build.rs ./build.rs

# Copy Cargo files, source code, and benches
COPY services/api-gateway/Cargo.toml ./
COPY services/api-gateway/src ./src
COPY services/api-gateway/benches ./benches

# Build the application (this will compile protos via build.rs)
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/api-gateway /usr/local/bin/app

# Create config and openapi directories
RUN mkdir -p /app/config /app/services/api-gateway/openapi

# Copy default gateway configuration file
COPY config/gateway-config.docker.yaml /app/config/gateway-config.yaml

# Copy OpenAPI documentation
COPY services/api-gateway/openapi/openapi.yaml /app/services/api-gateway/openapi/openapi.yaml

EXPOSE 8080
CMD ["app"]
