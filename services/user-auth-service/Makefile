.PHONY: proto build run test clean docker-build

# Generate protobuf code
proto:
	@echo "Generating protobuf code..."
	@mkdir -p api/proto
	PATH=$$PATH:$(shell go env GOPATH)/bin protoc --go_out=. --go_opt=paths=source_relative \
		--go-grpc_out=. --go-grpc_opt=paths=source_relative \
		-I ../../proto \
		../../proto/user.proto \
		../../proto/auth.proto
	@mkdir -p api/proto/authpb
	@mv -f user.pb.go user_grpc.pb.go api/proto/ 2>/dev/null || true
	@mv -f auth.pb.go auth_grpc.pb.go api/proto/authpb/ 2>/dev/null || true

# Build the service
build: proto
	@echo "Building user-auth-service..."
	go build -o bin/server cmd/server/main.go

# Run the service
run: build
	@echo "Running user-auth-service..."
	./bin/server

# Run tests
test:
	go test -v ./...

# Clean build artifacts
clean:
	rm -rf bin/
	rm -rf api/proto/*.pb.go

# Build Docker image
docker-build:
	docker build -t user-auth-service:latest .

# Install dependencies
deps:
	go mod download
	go mod tidy
