package health

import (
	"context"
	"database/sql"
	"time"

	"google.golang.org/grpc/health/grpc_health_v1"
)

// HealthChecker implements the gRPC health check service
type HealthChecker struct {
	grpc_health_v1.UnimplementedHealthServer
	db *sql.DB
}

// NewHealthChecker creates a new health checker with database dependency
func NewHealthChecker(db *sql.DB) *HealthChecker {
	return &HealthChecker{
		db: db,
	}
}

// Check performs a health check
func (h *HealthChecker) Check(ctx context.Context, req *grpc_health_v1.HealthCheckRequest) (*grpc_health_v1.HealthCheckResponse, error) {
	// Create a context with 2-second timeout for health check
	checkCtx, cancel := context.WithTimeout(ctx, 2*time.Second)
	defer cancel()

	// Check database connectivity
	if err := h.checkDatabase(checkCtx); err != nil {
		return &grpc_health_v1.HealthCheckResponse{
			Status: grpc_health_v1.HealthCheckResponse_NOT_SERVING,
		}, nil
	}

	return &grpc_health_v1.HealthCheckResponse{
		Status: grpc_health_v1.HealthCheckResponse_SERVING,
	}, nil
}

// Watch performs a streaming health check
func (h *HealthChecker) Watch(req *grpc_health_v1.HealthCheckRequest, stream grpc_health_v1.Health_WatchServer) error {
	// Send initial status
	ctx := stream.Context()

	// Create a context with 2-second timeout for health check
	checkCtx, cancel := context.WithTimeout(ctx, 2*time.Second)
	defer cancel()

	status := grpc_health_v1.HealthCheckResponse_SERVING
	if err := h.checkDatabase(checkCtx); err != nil {
		status = grpc_health_v1.HealthCheckResponse_NOT_SERVING
	}

	if err := stream.Send(&grpc_health_v1.HealthCheckResponse{
		Status: status,
	}); err != nil {
		return err
	}

	// Keep the stream open and send periodic updates
	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-ticker.C:
			// Check health again
			checkCtx, cancel := context.WithTimeout(ctx, 2*time.Second)
			status := grpc_health_v1.HealthCheckResponse_SERVING
			if err := h.checkDatabase(checkCtx); err != nil {
				status = grpc_health_v1.HealthCheckResponse_NOT_SERVING
			}
			cancel()

			if err := stream.Send(&grpc_health_v1.HealthCheckResponse{
				Status: status,
			}); err != nil {
				return err
			}
		}
	}
}

// checkDatabase verifies PostgreSQL connectivity
func (h *HealthChecker) checkDatabase(ctx context.Context) error {
	// Ping the database with context
	if err := h.db.PingContext(ctx); err != nil {
		return err
	}
	return nil
}
