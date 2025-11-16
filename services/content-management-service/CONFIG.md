# Content Management Service Configuration

This document describes the configuration system for the Content Management Service.

## Configuration Methods

The service supports three methods of configuration (in order of precedence):

1. **Environment Variables** (highest priority)
2. **YAML Configuration File** (medium priority)
3. **Default Values** (lowest priority)

## Environment Variables

Environment variables use double underscores (`__`) as separators for nested configuration.

Example:
```bash
SERVER__PORT=8082
DATABASE__MAX_CONNECTIONS=20
S3__ENDPOINT=http://localhost:9000
```

See `.env.example` for a complete list of available environment variables.

## YAML Configuration File

To use a YAML configuration file:

1. Copy `config.example.yaml` to `config.yaml`
2. Update the values as needed
3. Set the `CONFIG_FILE` environment variable:
   ```bash
   export CONFIG_FILE=config.yaml
   ```

## Configuration Sections

### Server Configuration

Controls the HTTP, gRPC, and metrics server settings.

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER__HOST` | `0.0.0.0` | Server bind address |
| `SERVER__PORT` | `8082` | HTTP server port |
| `SERVER__GRPC_PORT` | `50052` | gRPC server port |
| `SERVER__METRICS_PORT` | `9092` | Prometheus metrics port |
| `SERVER__SHUTDOWN_TIMEOUT_SECONDS` | `30` | Graceful shutdown timeout |

### Database Configuration

PostgreSQL database connection settings.

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE__URL` | `postgresql://...` | PostgreSQL connection URL |
| `DATABASE__MAX_CONNECTIONS` | `20` | Maximum connection pool size |
| `DATABASE__MIN_CONNECTIONS` | `5` | Minimum connection pool size |
| `DATABASE__CONNECTION_TIMEOUT_SECONDS` | `30` | Connection timeout |
| `DATABASE__IDLE_TIMEOUT_SECONDS` | `600` | Idle connection timeout |
| `DATABASE__MAX_LIFETIME_SECONDS` | `1800` | Maximum connection lifetime |

### S3/MinIO Configuration

Object storage settings for file uploads.

| Variable | Default | Description |
|----------|---------|-------------|
| `S3__ENDPOINT` | `http://localhost:9000` | S3-compatible endpoint |
| `S3__ACCESS_KEY` | `minioadmin` | S3 access key |
| `S3__SECRET_KEY` | `minioadmin` | S3 secret key |
| `S3__BUCKET` | `content-storage` | S3 bucket name |
| `S3__REGION` | `us-east-1` | S3 region |
| `S3__USE_PATH_STYLE` | `true` | Use path-style URLs (for MinIO) |
| `S3__PRESIGNED_URL_EXPIRY_SECONDS` | `3600` | Document download URL expiry (1 hour) |
| `S3__VIDEO_PRESIGNED_URL_EXPIRY_SECONDS` | `7200` | Video download URL expiry (2 hours) |

### ElasticSearch Configuration

Full-text search engine settings.

| Variable | Default | Description |
|----------|---------|-------------|
| `ELASTICSEARCH__URL` | `http://localhost:9200` | ElasticSearch endpoint |
| `ELASTICSEARCH__INDEX` | `content` | Index name for content |
| `ELASTICSEARCH__MAX_RETRIES` | `2` | Maximum retry attempts |
| `ELASTICSEARCH__TIMEOUT_SECONDS` | `30` | Request timeout |

### Redis Configuration

Job queue and caching settings.

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS__URL` | `redis://localhost:6379` | Redis connection URL |
| `REDIS__QUEUE_NAME` | `transcoding_jobs` | Queue name for transcoding jobs |
| `REDIS__CONNECTION_TIMEOUT_SECONDS` | `5` | Connection timeout |
| `REDIS__MAX_POOL_SIZE` | `10` | Maximum connection pool size |

### Observability Configuration

OpenTelemetry, Prometheus, and logging settings.

| Variable | Default | Description |
|----------|---------|-------------|
| `OBSERVABILITY__SERVICE_NAME` | `content-management-service` | Service name for tracing |
| `OBSERVABILITY__OTLP_ENDPOINT` | `http://localhost:4317` | OpenTelemetry collector endpoint |
| `OBSERVABILITY__LOG_LEVEL` | `info` | Log level (error, warn, info, debug, trace) |
| `OBSERVABILITY__ENABLE_TRACING` | `true` | Enable distributed tracing |
| `OBSERVABILITY__ENABLE_METRICS` | `true` | Enable Prometheus metrics |
| `OBSERVABILITY__ENABLE_LOGGING` | `true` | Enable structured logging |

### Analytics Configuration

Analytics service integration settings.

| Variable | Default | Description |
|----------|---------|-------------|
| `ANALYTICS__SERVICE_URL` | `http://localhost:50053` | Analytics service gRPC endpoint |
| `ANALYTICS__BATCH_SIZE` | `100` | Events per batch |
| `ANALYTICS__BATCH_INTERVAL_SECONDS` | `30` | Batch send interval |
| `ANALYTICS__RETRY_INTERVAL_SECONDS` | `300` | Retry interval (5 minutes) |
| `ANALYTICS__MAX_QUEUE_AGE_HOURS` | `24` | Maximum event age before discard |

### Upload Configuration

File upload settings and restrictions.

| Variable | Default | Description |
|----------|---------|-------------|
| `UPLOAD__MAX_FILE_SIZE_BYTES` | `524288000` | Maximum file size (500MB) |
| `UPLOAD__CHUNK_SIZE_BYTES` | `5242880` | Chunk size for uploads (5MB) |
| `UPLOAD__SESSION_EXPIRY_HOURS` | `24` | Upload session expiry |
| `UPLOAD__ALLOWED_VIDEO_TYPES` | `["video/mp4",...]` | Allowed video MIME types |
| `UPLOAD__ALLOWED_DOCUMENT_TYPES` | `["application/pdf",...]` | Allowed document MIME types |
| `UPLOAD__ENABLE_MALWARE_SCAN` | `false` | Enable ClamAV malware scanning |

### Transcoding Configuration

Video transcoding worker settings.

| Variable | Default | Description |
|----------|---------|-------------|
| `TRANSCODING__WORKER_COUNT` | `2` | Number of transcoding workers |
| `TRANSCODING__MAX_RETRIES` | `3` | Maximum retry attempts |
| `TRANSCODING__SEGMENT_DURATION_SECONDS` | `6` | Video segment duration |
| `TRANSCODING__OUTPUT_FORMATS` | `["hls","dash"]` | Output formats |

**Bitrate Variants** (configured in YAML or code):
- 360p: 640x360 @ 800kbps
- 480p: 854x480 @ 1400kbps
- 720p: 1280x720 @ 2800kbps
- 1080p: 1920x1080 @ 5000kbps

## Configuration Validation

The service validates all configuration values on startup. If validation fails, the service will exit with an error message indicating the problem.

Common validation errors:
- Missing required values (database URL, S3 credentials, etc.)
- Invalid port numbers (must be > 0)
- Invalid connection pool settings (min > max)
- Empty allowed file type lists

## Docker Configuration

When running in Docker, use environment variables in `docker-compose.yml`:

```yaml
services:
  content-management-service:
    environment:
      DATABASE__URL: "postgresql://cms:cms_password@postgres-cms:5432/cms"
      S3__ENDPOINT: "http://minio:9000"
      ELASTICSEARCH__URL: "http://elasticsearch:9200"
      REDIS__URL: "redis://redis:6379"
      OBSERVABILITY__OTLP_ENDPOINT: "http://tempo:4317"
```

## Development vs Production

### Development
```bash
# Use .env file
cp .env.example .env
# Edit .env with local values
cargo run
```

### Production
```bash
# Set environment variables directly
export DATABASE__URL="postgresql://..."
export S3__ENDPOINT="https://s3.amazonaws.com"
# ... other production values
cargo run --release
```

## Security Considerations

- **Never commit** `.env` files or `config.yaml` with real credentials
- Use secrets management (AWS Secrets Manager, HashiCorp Vault) in production
- Rotate S3 credentials regularly
- Use TLS for all external connections in production
- Enable malware scanning (`UPLOAD__ENABLE_MALWARE_SCAN=true`) in production

## Troubleshooting

### Configuration not loading
- Check that environment variables use double underscores (`__`)
- Verify `CONFIG_FILE` path is correct if using YAML
- Check file permissions on config files

### Validation errors
- Review error message for specific validation failure
- Ensure all required fields are set
- Check that numeric values are within valid ranges

### Connection failures
- Verify service endpoints are reachable
- Check network connectivity and firewall rules
- Ensure dependent services (PostgreSQL, MinIO, etc.) are running
