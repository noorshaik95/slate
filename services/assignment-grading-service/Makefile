.PHONY: proto build run test clean docker-build

# Generate protobuf code
proto:
	@echo "Generating protobuf code..."
	@mkdir -p api/proto
	PATH=$$PATH:$(shell go env GOPATH)/bin protoc --go_out=. --go_opt=paths=source_relative \
		--go-grpc_out=. --go-grpc_opt=paths=source_relative \
		-I ../../proto \
		../../proto/assignment.proto
	@mv -f assignment.pb.go assignment_grpc.pb.go api/proto/ 2>/dev/null || true

# Build the service
build: proto
	@echo "Building assignment-grading-service..."
	go build -o bin/server cmd/server/main.go

# Run the service
run: build
	@echo "Running assignment-grading-service..."
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
	docker build -t assignment-grading-service:latest .

# Install dependencies
deps:
	go mod download
	go mod tidy
