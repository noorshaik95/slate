package health

import (
	"context"
	"database/sql"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/health/grpc_health_v1"
)

func TestNewHealthChecker(t *testing.T) {
	db, _, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	checker := NewHealthChecker(db)
	assert.NotNil(t, checker)
	assert.Equal(t, db, checker.db)
}

func TestCheck_Healthy(t *testing.T) {
	db, mock, err := sqlmock.New(sqlmock.MonitorPingsOption(true))
	require.NoError(t, err)
	defer db.Close()

	// Expect a ping
	mock.ExpectPing()

	checker := NewHealthChecker(db)
	req := &grpc_health_v1.HealthCheckRequest{}

	resp, err := checker.Check(context.Background(), req)
	require.NoError(t, err)
	assert.NotNil(t, resp)
	assert.Equal(t, grpc_health_v1.HealthCheckResponse_SERVING, resp.Status)

	// Verify all expectations were met
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestCheck_Unhealthy(t *testing.T) {
	db, mock, err := sqlmock.New(sqlmock.MonitorPingsOption(true))
	require.NoError(t, err)
	defer db.Close()

	// Expect a ping that fails
	mock.ExpectPing().WillReturnError(sql.ErrConnDone)

	checker := NewHealthChecker(db)
	req := &grpc_health_v1.HealthCheckRequest{}

	resp, err := checker.Check(context.Background(), req)
	require.NoError(t, err)
	assert.NotNil(t, resp)
	assert.Equal(t, grpc_health_v1.HealthCheckResponse_NOT_SERVING, resp.Status)

	// Verify all expectations were met
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestCheck_Timeout(t *testing.T) {
	db, mock, err := sqlmock.New(sqlmock.MonitorPingsOption(true))
	require.NoError(t, err)
	defer db.Close()

	// Expect a ping that takes too long
	mock.ExpectPing().WillDelayFor(3 * time.Second)

	checker := NewHealthChecker(db)
	req := &grpc_health_v1.HealthCheckRequest{}

	start := time.Now()
	resp, err := checker.Check(context.Background(), req)
	duration := time.Since(start)

	require.NoError(t, err)
	assert.NotNil(t, resp)
	assert.Equal(t, grpc_health_v1.HealthCheckResponse_NOT_SERVING, resp.Status)

	// Verify timeout occurred (should be around 2 seconds, not 3)
	assert.Less(t, duration, 3*time.Second)
	assert.GreaterOrEqual(t, duration, 2*time.Second)
}

func TestCheckDatabase_Success(t *testing.T) {
	db, mock, err := sqlmock.New(sqlmock.MonitorPingsOption(true))
	require.NoError(t, err)
	defer db.Close()

	mock.ExpectPing()

	checker := NewHealthChecker(db)
	err = checker.checkDatabase(context.Background())
	assert.NoError(t, err)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestCheckDatabase_Failure(t *testing.T) {
	db, mock, err := sqlmock.New(sqlmock.MonitorPingsOption(true))
	require.NoError(t, err)
	defer db.Close()

	mock.ExpectPing().WillReturnError(sql.ErrConnDone)

	checker := NewHealthChecker(db)
	err = checker.checkDatabase(context.Background())
	assert.Error(t, err)
	assert.Equal(t, sql.ErrConnDone, err)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestCheckDatabase_ContextCancellation(t *testing.T) {
	db, mock, err := sqlmock.New(sqlmock.MonitorPingsOption(true))
	require.NoError(t, err)
	defer db.Close()

	// Expect a ping that takes a while
	mock.ExpectPing().WillDelayFor(1 * time.Second)

	checker := NewHealthChecker(db)

	// Create a context that cancels immediately
	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	err = checker.checkDatabase(ctx)
	assert.Error(t, err)
}

// Mock stream for testing Watch
type mockHealthWatchServer struct {
	grpc_health_v1.Health_WatchServer
	ctx      context.Context
	sentMsgs []*grpc_health_v1.HealthCheckResponse
}

func (m *mockHealthWatchServer) Send(resp *grpc_health_v1.HealthCheckResponse) error {
	m.sentMsgs = append(m.sentMsgs, resp)
	return nil
}

func (m *mockHealthWatchServer) Context() context.Context {
	return m.ctx
}

func TestWatch_InitialStatus(t *testing.T) {
	db, mock, err := sqlmock.New(sqlmock.MonitorPingsOption(true))
	require.NoError(t, err)
	defer db.Close()

	// Expect a ping for initial check
	mock.ExpectPing()

	checker := NewHealthChecker(db)
	req := &grpc_health_v1.HealthCheckRequest{}

	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()

	stream := &mockHealthWatchServer{
		ctx:      ctx,
		sentMsgs: make([]*grpc_health_v1.HealthCheckResponse, 0),
	}

	// Watch will block until context is cancelled
	err = checker.Watch(req, stream)

	// Should have sent at least one message
	assert.GreaterOrEqual(t, len(stream.sentMsgs), 1)
	assert.Equal(t, grpc_health_v1.HealthCheckResponse_SERVING, stream.sentMsgs[0].Status)
}

func TestWatch_UnhealthyStatus(t *testing.T) {
	db, mock, err := sqlmock.New(sqlmock.MonitorPingsOption(true))
	require.NoError(t, err)
	defer db.Close()

	// Expect a ping that fails
	mock.ExpectPing().WillReturnError(sql.ErrConnDone)

	checker := NewHealthChecker(db)
	req := &grpc_health_v1.HealthCheckRequest{}

	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()

	stream := &mockHealthWatchServer{
		ctx:      ctx,
		sentMsgs: make([]*grpc_health_v1.HealthCheckResponse, 0),
	}

	// Watch will block until context is cancelled
	err = checker.Watch(req, stream)

	// Should have sent at least one message
	assert.GreaterOrEqual(t, len(stream.sentMsgs), 1)
	assert.Equal(t, grpc_health_v1.HealthCheckResponse_NOT_SERVING, stream.sentMsgs[0].Status)
}
