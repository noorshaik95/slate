package main

import (
	"context"
	"net"
	"net/http"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	pb "github.com/noorshaik95/axum-grafana-example/services/user-auth-service/api/proto"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/config"
	grpcHandler "github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/grpc"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/health"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/repository"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/service"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/migrations"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/pkg/database"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/pkg/jwt"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/pkg/logger"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/pkg/metrics"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/pkg/ratelimit"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/pkg/tracing"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
	"google.golang.org/grpc"
	"google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/reflection"
)

func main() {
	// Initialize logger with level from environment
	logLevel := os.Getenv("LOG_LEVEL")
	if logLevel == "" {
		logLevel = "info" // Default to info level
	}
	log := logger.NewLogger(logLevel)
	log.Info().Str("log_level", logLevel).Msg("Starting User Auth Service")

	// Load configuration
	cfg, err := config.Load()
	if err != nil {
		log.Error().Err(err).Msg("Failed to load configuration")
		os.Exit(1)
	}

	// Initialize OpenTelemetry tracing
	otlpEndpoint := os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT")
	if otlpEndpoint == "" {
		otlpEndpoint = "tempo:4317" // Default to Tempo
	}

	tracingCfg := tracing.Config{
		ServiceName:    "user-auth-service",
		ServiceVersion: "1.0.0",
		OTLPEndpoint:   otlpEndpoint,
		OTLPInsecure:   true, // Use insecure for local development
		SamplingRate:   1.0,  // Sample all traces
	}

	log.Info().Str("otlp_endpoint", otlpEndpoint).Msg("Initializing OpenTelemetry tracing")
	tp, err := tracing.InitTracer(tracingCfg)
	if err != nil {
		log.Error().Err(err).Msg("Failed to initialize tracing")
		log.Info().Msg("Continuing without tracing")
	} else {
		log.Info().Msg("OpenTelemetry tracing initialized successfully")
		defer func() {
			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()
			if err := tracing.Shutdown(ctx, tp); err != nil {
				log.Error().Err(err).Msg("Failed to shutdown tracer provider")
			}
		}()
	}

	// Connect to database
	db, err := database.NewPostgresDB(cfg.Database.DSN())
	if err != nil {
		log.Error().Err(err).Msg("Failed to connect to database")
		os.Exit(1)
	}
	defer db.Close()

	log.Info().Msg("Connected to PostgreSQL database")

	// Run migrations
	migrationsPath := filepath.Join(".", "migrations")
	if _, err := os.Stat(migrationsPath); os.IsNotExist(err) {
		// Try alternative path for Docker
		migrationsPath = "/app/migrations"
	}

	if err := migrations.RunMigrations(db.DB, migrationsPath); err != nil {
		log.Error().Err(err).Msg("Failed to run migrations")
		os.Exit(1)
	}

	// Initialize repositories
	userRepo := repository.NewUserRepository(db.DB)
	roleRepo := repository.NewRoleRepository(db.DB)

	// Ensure default roles exist
	if err := roleRepo.EnsureDefaultRoles(context.Background()); err != nil {
		log.Error().Err(err).Msg("Failed to ensure default roles")
		os.Exit(1)
	}

	// Initialize JWT service
	tokenService := jwt.NewTokenService(
		cfg.JWT.SecretKey,
		cfg.JWT.AccessTokenDuration,
		cfg.JWT.RefreshTokenDuration,
	)
	tokenServiceAdapter := jwt.NewTokenServiceAdapter(tokenService)

	// Initialize Prometheus metrics
	registry := prometheus.NewRegistry()
	metricsCollector := metrics.NewMetrics(registry)
	log.Info().Msg("Prometheus metrics initialized")

	// Start metrics HTTP server
	metricsAddr := ":9090"
	metricsServer := &http.Server{
		Addr:    metricsAddr,
		Handler: promhttp.HandlerFor(registry, promhttp.HandlerOpts{Registry: registry}),
	}

	go func() {
		log.Info().Str("address", metricsAddr).Msg("Starting metrics HTTP server")
		if err := metricsServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Error().Err(err).Msg("Metrics server failed")
		}
	}()

	// Initialize services
	userService := service.NewUserService(userRepo, roleRepo, tokenServiceAdapter, log, metricsCollector)

	// Initialize rate limiter (Redis-based)
	var rateLimiter *ratelimit.RedisRateLimiter
	redisHost := os.Getenv("REDIS_HOST")
	redisPort := os.Getenv("REDIS_PORT")
	rateLimitEnabled := os.Getenv("RATE_LIMIT_ENABLED")

	if rateLimitEnabled == "true" && redisHost != "" && redisPort != "" {
		redisAddr := redisHost + ":" + redisPort
		log.Info().Str("redis_addr", redisAddr).Msg("Initializing Redis rate limiter")

		// Configure rate limits from environment or use defaults
		loginLimit := ratelimit.RateLimit{
			MaxAttempts: 5,                // 5 attempts
			Window:      15 * time.Minute, // per 15 minutes
		}
		registerLimit := ratelimit.RateLimit{
			MaxAttempts: 3,             // 3 attempts
			Window:      1 * time.Hour, // per hour
		}

		rateLimiter, err = ratelimit.NewRedisRateLimiter(redisAddr, loginLimit, registerLimit)
		if err != nil {
			log.Error().Err(err).Msg("Failed to initialize rate limiter")
			log.Info().Msg("Continuing without rate limiting")
		} else {
			log.Info().Msg("Rate limiter initialized successfully")
			defer rateLimiter.Close()
		}
	} else {
		log.Info().Msg("Rate limiting disabled (RATE_LIMIT_ENABLED not set or Redis not configured)")
	}

	// Initialize gRPC server with OpenTelemetry interceptors
	grpcServer := grpc.NewServer(
		grpc.StatsHandler(otelgrpc.NewServerHandler()), // OpenTelemetry stats handler (new API)
		grpc.ChainUnaryInterceptor(
			tracing.LoggingUnaryInterceptor(), // Debug interceptor
		),
	)

	// Register services
	userServiceServer := grpcHandler.NewUserServiceServer(userService, rateLimiter)
	pb.RegisterUserServiceServer(grpcServer, userServiceServer)

	// Register health check service
	healthChecker := health.NewHealthChecker(db.DB)
	grpc_health_v1.RegisterHealthServer(grpcServer, healthChecker)
	log.Info().Msg("gRPC health check service registered")

	// Enable reflection for debugging with tools like grpcurl
	reflection.Register(grpcServer)

	// Start gRPC server
	lis, err := net.Listen("tcp", cfg.GRPC.Address())
	if err != nil {
		log.Error().Err(err).Str("address", cfg.GRPC.Address()).Msg("Failed to listen")
		os.Exit(1)
	}

	log.Info().Str("address", cfg.GRPC.Address()).Msg("gRPC server listening")

	// Handle graceful shutdown
	go func() {
		sigint := make(chan os.Signal, 1)
		signal.Notify(sigint, os.Interrupt, syscall.SIGTERM)
		<-sigint

		log.Info().Msg("Shutting down servers")

		// Shutdown metrics server
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := metricsServer.Shutdown(shutdownCtx); err != nil {
			log.Error().Err(err).Msg("Failed to shutdown metrics server")
		}

		// Shutdown gRPC server
		grpcServer.GracefulStop()
	}()

	// Start serving
	if err := grpcServer.Serve(lis); err != nil {
		log.Error().Err(err).Msg("Failed to serve")
		os.Exit(1)
	}

	log.Info().Msg("Server stopped")
}
