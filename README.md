# Microservices Monorepo

A modern microservices architecture with API Gateway and User Authentication service, featuring complete observability with Grafana, Prometheus, Tempo, and Loki.

## 🏗️ Architecture Overview

This monorepo contains two main microservices:

1. **API Gateway** (Rust/Axum) - High-performance HTTP/gRPC gateway with auth, rate limiting, circuit breakers, and observability
2. **User Auth Service** (Go) - Complete user management with JWT authentication and RBAC

```
┌─────────────────────────────────────────────────────────────┐
│                        API Gateway (Rust)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Auth        │  │  Rate        │  │  Circuit     │     │
│  │  Middleware  │→ │  Limiter     │→ │  Breaker     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                           ↓
        ┌──────────────────┼──────────────────┐
        ↓                                      ↓
┌───────────────────────┐          ┌───────────────────────┐
│  User Auth Service    │          │  Other Backend        │
│  (Go/gRPC)            │          │  Services (gRPC)      │
│  ├─ Authentication    │          │  ├─ Business Logic    │
│  ├─ User CRUD         │          │  └─ ...               │
│  ├─ RBAC              │          └───────────────────────┘
│  └─ PostgreSQL        │
└───────────────────────┘
```

## 📁 Repository Structure

```
.
├── services/
│   ├── api-gateway/              # Rust-based API Gateway
│   │   ├── src/                  # Source code
│   │   │   ├── auth/             # Authentication & authorization
│   │   │   ├── circuit_breaker/  # Circuit breaker implementation
│   │   │   ├── handlers/         # HTTP handlers
│   │   │   └── ...
│   │   ├── Cargo.toml            # Rust dependencies
│   │   ├── Dockerfile            # Gateway Docker image
│   │   └── build.rs              # Proto build script
│   │
│   └── user-auth-service/        # Go-based User Auth Service
│       ├── cmd/server/           # Main server entry point
│       ├── internal/
│       │   ├── config/           # Configuration management
│       │   ├── grpc/             # gRPC handlers
│       │   ├── models/           # Data models
│       │   ├── repository/       # Database layer
│       │   └── service/          # Business logic
│       ├── pkg/
│       │   ├── database/         # Database utilities
│       │   ├── jwt/              # JWT token management
│       │   └── logger/           # Logging utilities
│       ├── migrations/           # SQL migrations
│       ├── api/proto/            # Generated protobuf code
│       ├── go.mod                # Go dependencies
│       ├── Dockerfile            # Auth service Docker image
│       └── Makefile              # Build automation
│
├── proto/                        # Shared protobuf definitions
│   ├── auth.proto               # Auth service proto
│   ├── service_auth.proto       # Service auth policy proto
│   └── user.proto               # User service proto (new)
│
├── config/                       # Shared configuration
│   ├── gateway-config.docker.yaml
│   ├── prometheus.yml
│   ├── tempo.yaml
│   ├── loki.yaml
│   └── grafana-datasources.yaml
│
├── docker-compose.yml            # Full stack orchestration
├── docker-compose.dev.yml        # Development environment
├── .env.example                  # Environment variables template
└── README.md                     # This file
```

## ✨ Features

### User Auth Service (Go)
- ✅ **User Management**: Complete CRUD operations
- ✅ **Authentication**: JWT-based with access/refresh tokens
- ✅ **Authorization**: Role-Based Access Control (RBAC)
- ✅ **Profile Management**: User profile updates
- ✅ **PostgreSQL**: Persistent storage with migrations
- ✅ **gRPC API**: High-performance gRPC server
- ✅ **Security**: Bcrypt password hashing, secure tokens

### API Gateway (Rust)
- ✅ **Dynamic Route Discovery**: Auto-discover routes from gRPC reflection
- ✅ **Protocol Translation**: HTTP/REST ↔ gRPC conversion with dynamic dispatch
- ✅ **Authentication**: JWT validation via user-auth-service
- ✅ **Connection Pooling**: Round-robin connection pool for high throughput
- ✅ **Rate Limiting**: Per-IP rate limiting with sliding window and automatic cleanup
- ✅ **Circuit Breaker**: Prevent cascading failures with state machine pattern
- ✅ **Request Timeouts**: Configurable timeouts to prevent resource exhaustion
- ✅ **Security**: Path traversal protection, body size limits, CORS support
- ✅ **Health Checks**: Separate liveness and readiness probes for Kubernetes
- ✅ **Observability**: Metrics, distributed tracing (W3C), and structured logging
- ✅ **High Performance**: Built with Axum and Tokio, production-ready

### Observability Stack
- **Grafana**: Visualization and dashboards
- **Prometheus**: Metrics collection
- **Tempo**: Distributed tracing
- **Loki**: Log aggregation
- **Promtail**: Log shipping

## 🚀 Quick Start

### Prerequisites
- Docker & Docker Compose
- Go 1.21+ (for local development)
- Rust 1.70+ (for local development)
- Protocol Buffers compiler (protoc)

### Run Full Stack

```bash
# Clone the repository
git clone <repository-url>
cd axum-grafana-example

# Copy environment variables
cp .env.example .env

# Start all services
docker-compose up --build

# Or run in detached mode
docker-compose up -d --build
```

### Access Services

- **API Gateway**: http://localhost:8080
- **User Auth Service (gRPC)**: localhost:50051
- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **PostgreSQL**: localhost:5432

## 🛠️ Development

### Running Services Individually

#### 1. Start Infrastructure Only

```bash
# Start PostgreSQL and observability stack
docker-compose -f docker-compose.dev.yml up -d
```

#### 2. Run User Auth Service Locally

```bash
cd services/user-auth-service

# Install dependencies
go mod download

# Generate protobuf code
make proto

# Run the service
make run

# Or build and run separately
make build
./bin/server
```

**Environment Variables:**
```bash
DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_PASSWORD=postgres
DB_NAME=userauth
JWT_SECRET=your-secret-key
GRPC_PORT=50051
```

#### 3. Run API Gateway Locally

```bash
cd services/api-gateway

# Build and run
GATEWAY_AUTH_SERVICE_ENDPOINT=http://localhost:50051 cargo run

# Or with release optimizations
cargo build --release
./target/release/api-gateway
```

## 📡 API Documentation

### User Auth Service (gRPC)

#### Authentication Endpoints

**Register**
```bash
grpcurl -plaintext -d '{
  "email": "user@example.com",
  "password": "password123",
  "first_name": "John",
  "last_name": "Doe",
  "phone": "+1234567890"
}' localhost:50051 user.UserService/Register
```

**Login**
```bash
grpcurl -plaintext -d '{
  "email": "user@example.com",
  "password": "password123"
}' localhost:50051 user.UserService/Login
```

**Validate Token**
```bash
grpcurl -plaintext -d '{
  "token": "your-jwt-token"
}' localhost:50051 user.UserService/ValidateToken
```

**Refresh Token**
```bash
grpcurl -plaintext -d '{
  "refresh_token": "your-refresh-token"
}' localhost:50051 user.UserService/RefreshToken
```

#### User Management Endpoints

- **CreateUser**: Create new user (admin)
- **GetUser**: Get user by ID
- **UpdateUser**: Update user details
- **DeleteUser**: Soft delete user
- **ListUsers**: List users with pagination

#### Profile Management

- **GetProfile**: Get user profile
- **UpdateProfile**: Update profile details
- **ChangePassword**: Change user password

#### Role Management (RBAC)

- **AssignRole**: Assign role to user
- **RemoveRole**: Remove role from user
- **GetUserRoles**: Get all user roles
- **CheckPermission**: Check user permission

### Default Roles

| Role | Permissions |
|------|-------------|
| **admin** | users.*, roles.*, system.manage |
| **user** | profile.read, profile.update |
| **manager** | users.read, users.update, profile.* |

### Default Admin User
- **Email**: `admin@example.com`
- **Password**: `admin123` ⚠️ **Change in production!**

## 🗄️ Database Schema

### Users
```sql
CREATE TABLE users (
    id VARCHAR(36) PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    phone VARCHAR(20),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

### Roles
```sql
CREATE TABLE roles (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    permissions TEXT[],
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

### User_Roles
```sql
CREATE TABLE user_roles (
    user_id VARCHAR(36) REFERENCES users(id),
    role_id VARCHAR(36) REFERENCES roles(id),
    assigned_at TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id, role_id)
);
```

## 📊 Observability

### Metrics

**API Gateway** (http://localhost:8080/metrics):
- `api_gateway_requests_total` - Total requests
- `api_gateway_request_duration_seconds` - Request latency
- `api_gateway_grpc_calls_total` - gRPC calls
- `api_gateway_circuit_breaker_state_changes_total` - Circuit breaker events
- `gateway_auth_failures_total` - Auth failures
- `gateway_rate_limit_exceeded_total` - Rate limit rejections

### Distributed Tracing

**Full end-to-end tracing** from API Gateway through to backend services using OpenTelemetry and Grafana Tempo.

**Quick Start**:
```bash
# Generate traces
./test_distributed_tracing.sh

# Or manually
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!","first_name":"Test","last_name":"User","phone":"+1234567890"}'
```

**View in Grafana** (http://localhost:3000):
1. Navigate to Explore → Select "Tempo"
2. Use TraceQL queries:
   ```traceql
   # All traces
   { }
   
   # By service
   { resource.service.name = "api-gateway" }
   { resource.service.name = "user-auth-service" }
   
   # By operation
   { span.name = "user.UserService/Register" }
   
   # Errors only
   { status = error }
   ```

**Features**:
- ✅ W3C Trace Context propagation
- ✅ Request flow visualization across services
- ✅ Latency breakdown by service and operation
- ✅ Error propagation and debugging
- ✅ Service dependency mapping

**Documentation**:
- Quick Start: `TRACING_QUICK_START.md`
- Full Setup: `DISTRIBUTED_TRACING_SETUP.md`
- Implementation: `TRACING_IMPLEMENTATION_SUMMARY.md`

### Logs

```bash
# View all logs
docker-compose logs -f

# View specific service
docker-compose logs -f user-auth-service
docker-compose logs -f api-gateway

# Query in Loki/Grafana
{container="user-auth-service"}
{container="api-gateway"}
```

## 🔒 Security

### Production Checklist

- [ ] Change default admin password
- [ ] Update `JWT_SECRET` to strong random value (32+ chars)
- [ ] Set strong PostgreSQL password
- [ ] Enable TLS for gRPC connections
- [ ] Configure CORS with specific origins
- [ ] Enable rate limiting with appropriate thresholds
- [ ] Set up monitoring alerts
- [ ] Review and restrict role permissions
- [ ] Implement token blacklist for logout
- [ ] Use HTTPS in production
- [ ] Regular security audits

### JWT Configuration

- **Access Token**: 15 minutes (default)
- **Refresh Token**: 7 days (default)
- **Algorithm**: HS256
- **Claims**: user_id, email, roles

## 🧪 Testing

### Test User Auth Service

```bash
cd services/user-auth-service
go test -v ./...
```

### Test API Gateway

```bash
cd services/api-gateway
cargo test
```

### Smoke Tests

Run end-to-end smoke tests to verify the full stack:

```bash
# Start the full stack first
docker-compose up -d

# Wait for services to be ready
sleep 10

# Run smoke tests
./tests/smoke_test.sh
```

The smoke tests verify:
- ✅ Gateway health endpoints (liveness/readiness)
- ✅ Metrics endpoint
- ✅ User registration and login flows
- ✅ Authenticated API calls
- ✅ CORS headers (if enabled)
- ✅ Rate limiting
- ✅ Path traversal protection

### Integration Testing with grpcurl

```bash
# Install grpcurl
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

# List services
grpcurl -plaintext localhost:50051 list

# Describe service
grpcurl -plaintext localhost:50051 describe user.UserService
```

## 🐛 Troubleshooting

### Database Connection Issues

```bash
# Check PostgreSQL is running
docker-compose ps postgres

# Test connection
psql -h localhost -U postgres -d userauth

# View logs
docker-compose logs postgres
```

### gRPC Connection Issues

```bash
# Test gRPC is accessible
grpcurl -plaintext localhost:50051 list

# Check service logs
docker-compose logs user-auth-service
```

### Service Won't Start

```bash
# Clean rebuild
docker-compose down -v
docker-compose up --build

# Check for port conflicts
lsof -i :8080  # API Gateway
lsof -i :50051 # User Auth
lsof -i :5432  # PostgreSQL
```

### Proto Generation Issues

```bash
cd services/user-auth-service

# Install protoc plugins
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

# Regenerate proto code
make proto
```

## 🔧 Building for Production

```bash
# Build all services
docker-compose build

# Build specific service
docker-compose build user-auth-service
docker-compose build api-gateway

# Run in production mode
docker-compose up -d

# View service status
docker-compose ps
```

## 📚 Additional Documentation

- [API Gateway Details](services/api-gateway/README.md) - Detailed gateway documentation
- [Proto Definitions](proto/) - gRPC service definitions
- [Migrations](services/user-auth-service/migrations/) - Database schema

## 🤝 Contributing

1. Create a feature branch
2. Make your changes
3. Run tests
4. Submit a pull request

## 📄 License

MIT License - See LICENSE file for details

## 🙏 Built With

- [Axum](https://github.com/tokio-rs/axum) - Web framework (Rust)
- [Gin](https://github.com/gin-gonic/gin) - Web framework (Go)
- [Tonic](https://github.com/hyperium/tonic) - gRPC (Rust)
- [gRPC-Go](https://github.com/grpc/grpc-go) - gRPC (Go)
- [PostgreSQL](https://www.postgresql.org/) - Database
- [OpenTelemetry](https://opentelemetry.io/) - Observability
- [Prometheus](https://prometheus.io/) - Metrics
- [Grafana](https://grafana.com/) - Visualization
- [Tempo](https://grafana.com/oss/tempo/) - Tracing
- [Loki](https://grafana.com/oss/loki/) - Logging
