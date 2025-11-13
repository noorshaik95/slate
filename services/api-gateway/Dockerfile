# Build Stage
FROM rust:1.90 as builder

# Install protobuf compiler for proto compilation
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy proto files for proto compilation (needed for build.rs)
COPY proto ./proto
COPY services/api-gateway/build.rs ./build.rs

# Copy Cargo files for dependency caching
COPY services/api-gateway/Cargo.toml services/api-gateway/Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached unless Cargo.toml/Cargo.lock changes)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY services/api-gateway/src ./src

# Build the application with actual source (this will compile protos via build.rs)
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/api-gateway /usr/local/bin/app

# Create config directory
RUN mkdir -p /app/config

# Copy default gateway configuration file
COPY config/gateway-config.docker.yaml /app/config/gateway-config.yaml

EXPOSE 8080
CMD ["app"]
