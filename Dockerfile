# Build Stage
FROM rust:1.90 as builder

# Install protobuf compiler for proto compilation
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy proto files first for proto compilation
COPY proto ./proto
COPY build.rs ./build.rs

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application (this will compile protos via build.rs)
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/api-gateway /usr/local/bin/app

# Create config directory
RUN mkdir -p /app/config

# Copy default gateway configuration file
COPY config/gateway-config.yaml /app/config/gateway-config.yaml

EXPOSE 8080
CMD ["app"]
