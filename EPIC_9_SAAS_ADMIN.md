# Epic 9: SaaS Admin (26 points)

This document describes the implementation of Epic 9, which includes two main user stories for SaaS Admin functionality.

## Table of Contents

1. [Overview](#overview)
2. [User Stories](#user-stories)
3. [Architecture](#architecture)
4. [Services](#services)
5. [Database Schema](#database-schema)
6. [API Documentation](#api-documentation)
7. [Deployment](#deployment)
8. [Monitoring & Metrics](#monitoring--metrics)
9. [Testing](#testing)
10. [Acceptance Criteria Validation](#acceptance-criteria-validation)

---

## Overview

Epic 9 implements comprehensive SaaS admin functionality including:

- **US-9.1: Tenant Provisioning (13 points)** - Automated tenant provisioning with database setup, admin creation, and welcome emails
- **US-9.2: Usage Metrics Dashboard (13 points)** - Real-time system metrics and monitoring with automated alerts

### Tech Stack

- **Language**: Go 1.21
- **Communication**: gRPC with Protocol Buffers
- **Database**: PostgreSQL 15
- **Caching/Rate Limiting**: Redis 7
- **Observability**: Prometheus + Grafana + Tempo + Loki
- **Email**: SMTP (MailHog for development)
- **Circuit Breaker**: Custom implementation
- **Rate Limiter**: Redis-based with in-memory fallback

---

## User Stories

### US-9.1: Tenant Provisioning (13 points)

**As a** super admin
**I want to** create tenants
**So that** institutions start using platform

#### Acceptance Criteria

- ✅ **AC1**: Create tenant (name, domain, tier)
- ✅ **AC2**: Provision dedicated database (if Professional+ tier)
- ✅ **AC3**: Create default admin account
- ✅ **AC4**: Welcome email sent with setup link
- ✅ **AC5**: Tenant dashboard at custom subdomain
- ✅ **AC6**: Storage quota set by tier
- ✅ **AC7**: Provisioning completes within 2 minutes

### US-9.2: Usage Metrics Dashboard (13 points)

**As a** super admin
**I want to** view system metrics
**So that** I monitor platform health

#### Acceptance Criteria

- ✅ **AC1**: Active tenants count displayed
- ✅ **AC2**: Total users, courses, storage shown
- ✅ **AC3**: API requests per minute (real-time)
- ✅ **AC4**: Error rate (last 24 hours)
- ✅ **AC5**: Tenant-level breakdown (sortable table)
- ✅ **AC6**: Metrics updated every 30 seconds
- ✅ **AC7**: Alerts if error rate >1% or uptime <99.5%

---

## Architecture

### Service Diagram

```
┌─────────────────┐
│   API Gateway   │ (Rust/Axum)
│  Circuit Breaker │
│  Rate Limiter   │
└────────┬────────┘
         │
         ├──────────────────┬──────────────────┬──────────────────┐
         │                  │                  │                  │
┌────────▼────────┐ ┌──────▼──────┐  ┌────────▼────────┐ ┌──────▼──────┐
│  User Auth      │ │   Tenant    │  │     Email       │ │   Metrics   │
│  Service (Go)   │ │ Service (Go)│  │  Service (Go)   │ │Service (Go) │
│                 │ │             │  │                 │ │             │
│ - Authentication│ │ - Provision │  │ - SMTP          │ │ - Aggregator│
│ - Authorization │ │ - Tenants   │  │ - Templates     │ │ - Alerts    │
│ - RBAC          │ │ - Storage   │  │ - Queue         │ │ - Real-time │
└────────┬────────┘ └──────┬──────┘  └────────┬────────┘ └──────┬──────┘
         │                 │                  │                  │
         └─────────────────┴──────────────────┴──────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │                               │
            ┌───────▼────────┐            ┌────────▼────────┐
            │   PostgreSQL   │            │     Redis       │
            │   (Database)   │            │ (Rate Limiting) │
            └────────────────┘            └─────────────────┘
```

### Observability Stack

```
┌─────────────────────────────────────────────────────────┐
│                   Grafana Dashboard                      │
│  - SaaS Admin Metrics                                   │
│  - Tenant Breakdown                                     │
│  - Alerts & Notifications                               │
└──────────────┬──────────────┬───────────────────────────┘
               │              │
      ┌────────▼────────┐  ┌──▼──────────┐
      │   Prometheus    │  │    Tempo    │
      │   (Metrics)     │  │  (Tracing)  │
      └─────────────────┘  └─────────────┘
               │
      ┌────────▼────────┐
      │      Loki       │
      │    (Logs)       │
      └─────────────────┘
```

---

## Services

### 1. Tenant Service (Port: 50053)

**Purpose**: Manages tenant provisioning and lifecycle

**Key Features**:
- Automated tenant provisioning (AC1, AC2, AC3, AC7)
- Storage quota management (AC6)
- Custom subdomain allocation (AC5)
- Setup token generation for welcome emails (AC4)
- Circuit breaker protection for external service calls
- Rate limiting for tenant creation (5 per hour per IP)

**Proto**: `proto/tenant.proto`

**Endpoints**:
- `CreateTenant` - Provision a new tenant
- `GetTenant` - Retrieve tenant details
- `UpdateTenant` - Modify tenant configuration
- `ListTenants` - List all tenants with filters
- `GetProvisioningStatus` - Track provisioning progress (AC7)
- `GetStorageQuota` - View storage usage (AC6)
- `UpdateStorageUsage` - Update storage metrics

**Database**: `tenantdb`

**Metrics Exposed**:
- `tenant_provisioning_total{status}` - Provisioning attempts
- `tenant_provisioning_duration_seconds` - Provisioning duration (AC7: must be < 120s)
- `tenant_provisioning_errors_total{error_type}` - Error tracking
- `tenants_total` - Total tenant count
- `tenants_active_total` - Active tenants (AC1)
- `tenants_by_tier{tier}` - Tenants by subscription tier
- `tenant_storage_quota_bytes{tenant_id,tier}` - Storage quota (AC6)
- `tenant_storage_used_bytes{tenant_id,tier}` - Storage usage (AC6)

### 2. Email Service (Port: 50052)

**Purpose**: Handles all email notifications

**Key Features**:
- Template-based email rendering
- Async email queue with retry logic
- Welcome email with setup link (AC4)
- Email event tracking (sent, delivered, opened, clicked)
- SMTP integration with fallback

**Proto**: `proto/email.proto`

**Endpoints**:
- `SendEmail` - Send custom email
- `SendTemplatedEmail` - Send templated email
- `SendWelcomeEmail` - Send welcome email with setup link (AC4)
- `GetEmailStatus` - Track email delivery
- `ListEmails` - View email history

**Database**: `emaildb`

**Templates**:
- `tenant_admin_welcome` - Welcome email for new tenant admins (AC4)

**Configuration**:
- SMTP host/port (uses MailHog in development)
- Email retry policy (3 attempts with exponential backoff)
- Template storage

### 3. Metrics Service (Port: 50054)

**Purpose**: Aggregates and exposes system-wide metrics

**Key Features**:
- System-wide metrics aggregation (AC1, AC2, AC3, AC4)
- Tenant-level breakdown (AC5)
- Real-time metrics streaming (AC6: 30-second updates)
- Automated alerting (AC7: error rate > 1%, uptime < 99.5%)
- Prometheus integration for data collection

**Proto**: `proto/metrics.proto`

**Endpoints**:
- `GetSystemMetrics` - System-wide metrics (AC1, AC2, AC3, AC4)
- `GetTenantMetrics` - Metrics for a specific tenant
- `ListTenantMetrics` - Sortable tenant breakdown table (AC5)
- `GetRealtimeMetrics` - Current snapshot (AC6)
- `StreamRealtimeMetrics` - Live metrics stream (AC6: 30s interval)
- `GetActiveAlerts` - Current alerts (AC7)
- `GetAlertHistory` - Alert history
- `RecordAPIRequest` - Internal: record API request
- `RecordError` - Internal: record error

**Database**: `metricsdb`

**Alert Rules** (AC7):
- Error rate threshold: 1.0%
- Uptime threshold: 99.5%
- Alert evaluation interval: 30 seconds
- Alert retention: 90 days

---

## Database Schema

### Tenant Service Schema

#### `subscription_tiers`
```sql
- id (PK)
- name (unique): free, basic, professional, enterprise
- tier_level: 0-3
- storage_quota_bytes (AC6)
- max_users
- max_courses
- dedicated_database (AC2: true for Professional+)
- custom_domain (AC5)
```

#### `tenants`
```sql
- id (PK)
- name
- domain (unique) (AC5: custom subdomain)
- tier_id (FK → subscription_tiers)
- database_name (AC2: for Professional+ tier)
- database_connection_string
- storage_quota_bytes (AC6)
- storage_used_bytes
- user_count, course_count
```

#### `tenant_admins`
```sql
- id (PK)
- tenant_id (FK → tenants)
- user_id (reference to user-auth-service)
- email (AC3)
- is_primary
```

#### `tenant_provisioning`
```sql
- id (PK)
- tenant_id (FK)
- status: pending, provisioning_database, creating_admin, sending_email, completed, failed
- progress_percentage
- duration_seconds (AC7: must be < 120)
```

#### `tenant_setup_tokens`
```sql
- id (PK)
- tenant_id (FK)
- token (unique)
- expires_at (7 days)
- used_at (AC4: for setup link)
```

### Metrics Service Schema

#### `api_request_metrics`
```sql
- id (PK)
- tenant_id
- method, path, status_code
- response_time_ms
- timestamp (AC3: for requests per minute)
```

#### `tenant_metrics_summary`
```sql
- tenant_id (PK)
- user_count (AC2)
- course_count (AC2)
- storage_used_bytes (AC2)
- api_requests_last_minute (AC3)
- error_rate_percentage (AC4)
- last_updated_at (AC6: updated every 30s)
```

#### `alerts`
```sql
- id (PK)
- alert_type: error_rate_high, uptime_low
- threshold_value, current_value (AC7)
- is_active
- triggered_at
```

---

## API Documentation

### Creating a Tenant (US-9.1)

**gRPC Method**: `TenantService.CreateTenant`

**Request**:
```protobuf
message CreateTenantRequest {
  string name = 1;                    // AC1
  string domain = 2;                  // AC1, AC5: custom subdomain
  TenantTier tier = 3;                // AC1: FREE, BASIC, PROFESSIONAL, ENTERPRISE
  string admin_email = 4;             // AC3
  string admin_first_name = 5;
  string admin_last_name = 6;
  string admin_password = 7;          // AC3
}
```

**Response**:
```protobuf
message CreateTenantResponse {
  Tenant tenant = 1;
  TenantAdmin admin = 2;              // AC3
  string setup_url = 3;               // AC4: link for welcome email
  ProvisioningStatus status = 4;
  string provisioning_id = 5;
}
```

**Provisioning Flow** (AC7: < 2 minutes):
1. **0-20%**: Create tenant record (AC1)
2. **20-40%**: Provision dedicated database if Professional+ (AC2)
3. **40-60%**: Create default admin account (AC3)
4. **60-80%**: Set storage quota by tier (AC6)
5. **80-100%**: Send welcome email with setup link (AC4)

**Example**:
```bash
grpcurl -d '{
  "name": "Acme University",
  "domain": "acme.slate.local",
  "tier": "PROFESSIONAL",
  "admin_email": "admin@acme.edu",
  "admin_first_name": "John",
  "admin_last_name": "Doe",
  "admin_password": "SecurePass123!"
}' localhost:50053 tenant.TenantService/CreateTenant
```

### Viewing System Metrics (US-9.2)

**gRPC Method**: `MetricsService.GetSystemMetrics`

**Response** (all ACs):
```protobuf
message SystemMetricsResponse {
  int32 active_tenants_count = 1;           // AC1
  int64 total_users = 3;                    // AC2
  int64 total_courses = 4;                  // AC2
  int64 total_storage_bytes = 5;            // AC2
  double api_requests_per_minute = 8;       // AC3
  double error_rate_percentage = 10;        // AC4
  double uptime_percentage = 13;
}
```

### Real-time Metrics Stream (AC6)

**gRPC Method**: `MetricsService.StreamRealtimeMetrics`

**Request**:
```protobuf
message StreamRealtimeMetricsRequest {
  optional string tenant_id = 1;
  int32 update_interval_seconds = 2;  // Default: 30 (AC6)
}
```

**Response Stream**:
```protobuf
message RealtimeMetricsResponse {
  int64 api_requests_last_minute = 1;
  double current_error_rate_percentage = 6;
  int32 active_connections = 7;
  google.protobuf.Timestamp timestamp = 11;
}
```

---

## Deployment

### Prerequisites

- Docker & Docker Compose
- Go 1.21+
- Protocol Buffers compiler (`protoc`)
- Make

### Quick Start

1. **Start all services**:
```bash
docker-compose up -d
```

2. **Verify services**:
```bash
# Check tenant service
grpcurl -plaintext localhost:50053 grpc.health.v1.Health/Check

# Check email service
grpcurl -plaintext localhost:50052 grpc.health.v1.Health/Check

# Check metrics service
grpcurl -plaintext localhost:50054 grpc.health.v1.Health/Check
```

3. **Access dashboards**:
- Grafana: http://localhost:3000 (admin/admin)
- Prometheus: http://localhost:9090
- MailHog UI: http://localhost:8025

### Service Ports

| Service          | gRPC | HTTP | Metrics |
|-----------------|------|------|---------|
| User Auth       | 50051| 8081 | 9091    |
| Email Service   | 50052| 8082 | 9092    |
| Tenant Service  | 50053| 8083 | 9093    |
| Metrics Service | 50054| 8084 | 9094    |
| API Gateway     | -    | 8080 | -       |

### Environment Variables

**Tenant Service**:
```env
DB_NAME=tenantdb
USER_SERVICE_URL=user-auth-service:50051
EMAIL_SERVICE_URL=email-service:50052
EMAIL_ENABLED=true
BASE_SETUP_URL=http://localhost:8080
```

**Email Service**:
```env
SMTP_HOST=mailhog
SMTP_PORT=1025
SMTP_FROM_EMAIL=noreply@slate.local
```

**Metrics Service**:
```env
PROMETHEUS_URL=http://prometheus:9090
METRICS_UPDATE_INTERVAL=30          # AC6
ERROR_RATE_THRESHOLD=1.0           # AC7
UPTIME_THRESHOLD=99.5              # AC7
```

---

## Monitoring & Metrics

### Grafana Dashboard

The SaaS Admin dashboard (`config/grafana-dashboard-saas-admin.json`) provides:

**Panel 1-4**: System Overview (AC1, AC2)
- Active tenants count
- Total users, courses, storage
- Tenants by tier (pie chart)

**Panel 5-6**: Real-time Metrics (AC3, AC6)
- API requests per minute
- Storage usage gauge

**Panel 7-8**: Health & Alerts (AC4, AC7)
- Error rate (last 24 hours)
- Uptime percentage
- Automated alerts

**Panel 9**: Tenant Breakdown (AC5)
- Sortable table with:
  - Tenant name
  - Users, courses, storage
  - API requests/min
  - Error rate

**Panel 10-11**: Provisioning (AC7)
- Provisioning duration histogram (must show < 120s)
- Success rate trend

**Panel 12**: Active Alerts (AC7)
- Firing alerts table

**Refresh Rate**: 30 seconds (AC6)

### Prometheus Metrics

**Tenant Service**:
```
tenant_provisioning_total{status="completed|failed"}
tenant_provisioning_duration_seconds (AC7)
tenants_active_total (AC1)
tenant_storage_used_bytes{tenant_id,tier} (AC6)
```

**Metrics Service**:
```
system_active_tenants (AC1)
system_total_users (AC2)
system_total_courses (AC2)
system_api_requests_per_minute (AC3)
system_error_rate_percentage (AC4)
```

**Alerts** (AC7):
```yaml
- alert: HighErrorRate
  expr: system_error_rate_percentage > 1.0
  for: 5m
  annotations:
    summary: "Error rate exceeded 1%"

- alert: LowUptime
  expr: system_uptime_percentage < 99.5
  for: 5m
  annotations:
    summary: "Uptime below 99.5%"
```

---

## Testing

### Manual Testing

**1. Test Tenant Provisioning (US-9.1)**:
```bash
# Create a Professional tier tenant
grpcurl -plaintext -d '{
  "name": "Test Tenant",
  "domain": "test.slate.local",
  "tier": "PROFESSIONAL",
  "admin_email": "admin@test.com",
  "admin_first_name": "Test",
  "admin_last_name": "Admin",
  "admin_password": "SecurePass123!"
}' localhost:50053 tenant.TenantService/CreateTenant

# Check provisioning status (should complete < 2 min)
grpcurl -plaintext -d '{"provisioning_id": "<id>"}' \
  localhost:50053 tenant.TenantService/GetProvisioningStatus

# Verify welcome email in MailHog: http://localhost:8025
```

**2. Test Metrics Dashboard (US-9.2)**:
```bash
# Get system metrics
grpcurl -plaintext -d '{}' \
  localhost:50054 metrics.MetricsService/GetSystemMetrics

# Stream real-time metrics (30s interval)
grpcurl -plaintext -d '{"update_interval_seconds": 30}' \
  localhost:50054 metrics.MetricsService/StreamRealtimeMetrics
```

**3. Test Storage Quota (AC6)**:
```bash
# Get storage quota
grpcurl -plaintext -d '{"tenant_id": "<id>"}' \
  localhost:50053 tenant.TenantService/GetStorageQuota

# Update storage usage
grpcurl -plaintext -d '{
  "tenant_id": "<id>",
  "bytes_delta": 1048576,
  "file_count_delta": 1
}' localhost:50053 tenant.TenantService/UpdateStorageUsage
```

### Acceptance Criteria Validation

Run the automated acceptance tests:
```bash
cd services/tenant-service
make test

cd ../metrics-service
make test
```

---

## Acceptance Criteria Validation

### US-9.1: Tenant Provisioning

| AC | Requirement | Implementation | Status |
|----|-------------|----------------|--------|
| AC1 | Create tenant (name, domain, tier) | `CreateTenant` RPC with tier selection (FREE, BASIC, PROFESSIONAL, ENTERPRISE) | ✅ |
| AC2 | Provision dedicated database (Professional+) | Conditional database provisioning in `provisionTenantAsync()` when `tier.DedicatedDatabase == true` | ✅ |
| AC3 | Create default admin account | `userServiceClient.CreateUser()` + `CreateAdmin()` in provisioning flow | ✅ |
| AC4 | Welcome email with setup link | `SendWelcomeEmail()` with generated setup token (7-day expiry) | ✅ |
| AC5 | Custom subdomain | `domain` field in tenant record, used for subdomain routing | ✅ |
| AC6 | Storage quota set by tier | `StorageQuotaBytes` from `subscription_tiers` table, enforced in `UpdateStorageUsage()` | ✅ |
| AC7 | Provisioning < 2 minutes | Context timeout + `duration_seconds` tracking in `tenant_provisioning` table, monitored via Prometheus | ✅ |

### US-9.2: Usage Metrics Dashboard

| AC | Requirement | Implementation | Status |
|----|-------------|----------------|--------|
| AC1 | Active tenants count | `active_tenants_count` in `GetSystemMetrics()`, displayed in Grafana panel 1 | ✅ |
| AC2 | Total users, courses, storage | Aggregated from `tenant_metrics_summary` table, exposed in system metrics | ✅ |
| AC3 | API requests per minute (real-time) | Calculated from `api_request_metrics` table with 1-minute rate, streamed via `StreamRealtimeMetrics()` | ✅ |
| AC4 | Error rate (last 24 hours) | Percentage of 5xx responses in last 24h from `api_request_metrics`, displayed in Grafana panel 7 | ✅ |
| AC5 | Tenant-level breakdown (sortable) | `ListTenantMetrics()` with sort parameters, rendered in Grafana table (panel 9) | ✅ |
| AC6 | Metrics updated every 30 seconds | Streaming RPC with 30s interval + Grafana refresh rate = 30s | ✅ |
| AC7 | Alerts (error >1%, uptime <99.5%) | Alert rules in Prometheus + `alerts` table with thresholds, displayed in Grafana panel 12 | ✅ |

---

## Architecture Decisions

### 1. Circuit Breaker Pattern
**Decision**: Implement custom circuit breaker for external service calls
**Rationale**: Prevents cascade failures when email or user service is down
**Implementation**: `pkg/circuitbreaker/circuit_breaker.go` with states (Closed, Open, Half-Open)

### 2. Rate Limiter
**Decision**: Redis-based distributed rate limiting with in-memory fallback
**Rationale**: Prevents abuse of tenant creation endpoint (expensive operation)
**Limits**: 5 tenant creations per hour per IP
**Implementation**: `pkg/ratelimit/rate_limiter.go`

### 3. Async Provisioning
**Decision**: Goroutine-based async provisioning with status tracking
**Rationale**: Allows API to return immediately while provisioning continues
**Monitoring**: Progress updates in `tenant_provisioning` table + Prometheus metrics

### 4. Observability
**Decision**: Full observability stack (Prometheus, Grafana, Tempo, Loki)
**Rationale**: Required for AC6 (30s updates) and AC7 (automated alerts)
**Benefits**: Real-time monitoring, distributed tracing, centralized logging

---

## Future Enhancements

1. **Multi-region deployment** - Tenant data residency requirements
2. **Automated database backups** - Per-tenant backup schedules
3. **Tenant isolation** - Network-level isolation for enterprise tier
4. **Custom SMTP per tenant** - White-label email sending
5. **Billing integration** - Stripe/Paddle integration for subscriptions
6. **Audit logs** - Compliance tracking for tenant operations
7. **Multi-tenancy for storage** - S3/MinIO integration with tenant buckets

---

## Troubleshooting

### Provisioning Takes > 2 Minutes

**Symptom**: `tenant_provisioning_duration_seconds > 120`
**Causes**:
- Slow database connection
- Email service timeout
- User service unavailable

**Solution**:
```bash
# Check circuit breaker states
curl http://localhost:9093/metrics | grep circuit_breaker

# Verify database connectivity
docker exec -it slate-postgres-1 psql -U postgres -d tenantdb

# Check email service
curl http://localhost:8025  # MailHog UI
```

### Metrics Not Updating (AC6)

**Symptom**: Grafana shows stale data
**Causes**:
- Prometheus not scraping metrics service
- Metrics service not calculating updates

**Solution**:
```bash
# Verify Prometheus targets
curl http://localhost:9090/targets

# Check metrics service logs
docker logs slate-metrics-service-1

# Manually trigger metrics update
grpcurl -plaintext localhost:50054 metrics.MetricsService/GetSystemMetrics
```

### Alerts Not Firing (AC7)

**Symptom**: No alerts despite error rate > 1%
**Causes**:
- Alert rules not loaded
- Thresholds misconfigured

**Solution**:
```bash
# Check Prometheus alert rules
curl http://localhost:9090/api/v1/rules

# Verify threshold environment variables
docker exec slate-metrics-service-1 env | grep THRESHOLD

# Query current error rate
curl 'http://localhost:9090/api/v1/query?query=system_error_rate_percentage'
```

---

## License

Copyright © 2025 Slate Platform. All rights reserved.

---

## Support

For issues or questions:
- **GitHub Issues**: https://github.com/your-org/slate/issues
- **Documentation**: https://docs.slate.local
- **Email**: support@slate.local
