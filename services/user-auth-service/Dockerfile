# Build stage
FROM golang:1.21-alpine AS builder

# Install protoc and required tools
RUN apk add --no-cache protobuf protobuf-dev git make

# Install Go protobuf plugins
RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@latest && \
    go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

WORKDIR /build

# Copy go mod files
COPY go.mod go.sum ./
RUN go mod download

# Copy proto files
COPY ../../proto /build/proto

# Copy source code
COPY . .

# Generate protobuf code
RUN mkdir -p api/proto && \
    protoc --go_out=. --go_opt=paths=source_relative \
    --go-grpc_out=. --go-grpc_opt=paths=source_relative \
    -I /build/proto \
    /build/proto/user.proto

# Build the application
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o server cmd/server/main.go

# Runtime stage
FROM alpine:latest

RUN apk --no-cache add ca-certificates

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/server .
COPY --from=builder /build/migrations ./migrations

# Expose gRPC port
EXPOSE 50051

# Run the service
CMD ["./server"]
