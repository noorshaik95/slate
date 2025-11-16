# Content Management Service

A Rust-based microservice for managing educational content including videos, PDFs, and documents with support for hierarchical organization, video streaming, progress tracking, and full-text search.

## Features

- **Hierarchical Content Organization**: Organize content in modules → lessons → resources
- **Multi-format File Uploads**: Support for videos (MP4, MOV, AVI), PDFs, and DOCX files up to 500MB
- **Chunked Upload**: Efficient upload of large files with resume capability
- **Video Transcoding**: Automatic transcoding to HLS/DASH with adaptive bitrate (360p, 480p, 720p, 1080p)
- **Video Streaming**: Adaptive bitrate streaming with playback controls
- **Progress Tracking**: Track student completion and generate instructor reports
- **Full-text Search**: ElasticSearch-powered content search
- **Download Management**: Time-limited signed URLs with copyright controls
- **Observability**: Comprehensive metrics, tracing, and logging

## Architecture

The service is built with:
- **Axum**: Web framework for HTTP/REST endpoints
- **Tonic**: gRPC server for service-to-service communication
- **SQLx**: PostgreSQL database access with compile-time query verification
- **AWS SDK**: S3-compatible object storage (MinIO)
- **ElasticSearch**: Full-text search indexing
- **Redis**: Job queue for async transcoding
- **OpenTelemetry**: Distributed tracing and metrics

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and configure:

#### Server Configuration
- `SERVER_HOST`: Server bind address (default: 0.0.0.0)
- `SERVER_PORT`: HTTP server port (default: 8082)
- `GRPC_PORT`: gRPC server port (default: 50052)
- `METRICS_PORT`: Prometheus metrics port (default: 9092)

#### Database Configuration
- `DATABASE_URL`: PostgreSQL connection string
- `DB_MAX_CONNECTIONS`: Maximum database connections (default: 20)
- `DB_MIN_CONNECTIONS`: Minimum database connections (default: 5)

#### S3/MinIO Configuration
- `S3_ENDPOINT`: S3-compatible endpoint URL
- `S3_ACCESS_KEY`: S3 access key
- `S3_SECRET_KEY`: S3 secret key
- `S3_BUCKET`: Bucket name for content storage
- `S3_REGION`: AWS region (default: us-east-1)

#### ElasticSearch Configuration
- `ELASTICSEARCH_URL`: ElasticSearch endpoint URL
- `ELASTICSEARCH_INDEX`: Index name for content (default: content)

#### Redis Configuration
- `REDIS_URL`: Redis connection URL
- `REDIS_QUEUE_NAME`: Queue name for transcoding jobs

#### Upload Configuration
- `UPLOAD__MAX_FILE_SIZE_BYTES`: Maximum file size (default: 524288000 = 500MB)
- `UPLOAD__CHUNK_SIZE_BYTES`: Chunk size for uploads (default: 5242880 = 5MB)
- `UPLOAD__SESSION_EXPIRY_HOURS`: Upload session expiry (default: 24)

#### Transcoding Configuration
- `TRANSCODING__WORKER_COUNT`: Number of transcoding workers (default: 2)
- `TRANSCODING__MAX_RETRIES`: Maximum retry attempts (default: 3)

### Configuration File

Alternatively, use a YAML configuration file:

```yaml
server:
  host: "0.0.0.0"
  port: 8082
  grpc_port: 50052
  metrics_port: 9092

database:
  url: "postgresql://cms:password@localhost:5432/cms"
  max_connections: 20
  min_connections: 5

s3:
  endpoint: "http://localhost:9000"
  access_key: "minioadmin"
  secret_key: "minioadmin"
  bucket: "content-storage"
  region: "us-east-1"

elasticsearch:
  url: "http://localhost:9200"
  index: "content"

redis:
  url: "redis://localhost:6379"
  queue_name: "transcoding_jobs"
```

## Running the Service

### Development

```bash
# Install dependencies
cargo build

# Run database migrations
sqlx migrate run

# Start the service
cargo run
```

### Docker

```bash
# Build the Docker image
docker build -t content-management-service -f services/content-management-service/Dockerfile .

# Run with docker-compose
docker-compose up content-management-service
```

### Docker Compose

The service is configured in `docker-compose.yml` with all dependencies:

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f content-management-service

# Stop services
docker-compose down
```

## API Endpoints

The service exposes gRPC endpoints that are automatically mapped to HTTP/REST by the API Gateway:

### Content Management
- `POST /api/modules` - Create module
- `GET /api/modules` - List modules
- `GET /api/modules/:id` - Get module
- `PUT /api/modules/:id` - Update module
- `DELETE /api/modules/:id` - Delete module
- (Similar endpoints for lessons and resources)

### Custom Endpoints
- `GET /api/content/structure` - Get complete content hierarchy
- `POST /api/content/reorder` - Reorder content
- `POST /api/content/publish` - Publish content
- `POST /api/content/unpublish` - Unpublish content

### Upload Endpoints
- `POST /api/content/upload/initiate` - Initiate chunked upload
- `POST /api/content/upload/chunk` - Upload chunk
- `POST /api/content/upload/complete` - Complete upload
- `POST /api/content/upload/cancel` - Cancel upload

### Streaming Endpoints
- `GET /api/content/videos/:id/manifest` - Get video manifest
- `POST /api/content/videos/:id/position` - Update playback position
- `GET /api/content/videos/:id/playback` - Get playback state

### Progress Endpoints
- `POST /api/content/progress/complete` - Mark content complete
- `GET /api/content/progress` - Get student progress
- `GET /api/content/progress/report` - Get instructor report

### Search & Download
- `GET /api/content/search` - Search content
- `POST /api/content/download` - Generate download URL

## Health Checks

- **Liveness**: `GET http://localhost:8082/health/live`
- **Readiness**: `GET http://localhost:8082/health/ready`

## Metrics

Prometheus metrics are exposed at `http://localhost:9092/metrics`:

- `cms_uploads_total` - Total uploads by status
- `cms_upload_duration_seconds` - Upload duration histogram
- `cms_video_streams_total` - Total video streams
- `cms_search_queries_total` - Total search queries
- `cms_transcoding_jobs_total` - Total transcoding jobs
- `cms_db_connections_active` - Active database connections

## Tracing

Distributed traces are sent to Tempo via OpenTelemetry. View traces in Grafana at `http://localhost:3000`.

## Database Migrations

Migrations are located in `migrations/` and run automatically on startup. To run manually:

```bash
sqlx migrate run --database-url postgresql://cms:password@localhost:5432/cms
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Troubleshooting

### Database Connection Issues

Ensure PostgreSQL is running and accessible:
```bash
psql -h localhost -p 5432 -U cms -d cms
```

### MinIO Connection Issues

Check MinIO is running:
```bash
curl http://localhost:9000/minio/health/live
```

### ElasticSearch Connection Issues

Verify ElasticSearch is accessible:
```bash
curl http://localhost:9200/_cluster/health
```

### Transcoding Issues

Check FFmpeg is installed:
```bash
ffmpeg -version
```

View transcoding job logs:
```bash
docker-compose logs -f content-management-service | grep transcoding
```

## License

Copyright © 2025 Slate Platform
