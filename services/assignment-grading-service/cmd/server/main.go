package main

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	"slate/services/assignment-grading-service/internal/config"
	"slate/services/assignment-grading-service/internal/health"
	"slate/services/assignment-grading-service/internal/repository"
	"slate/services/assignment-grading-service/internal/service"
	"slate/services/assignment-grading-service/migrations"
	"slate/services/assignment-grading-service/pkg/database"
	"slate/services/assignment-grading-service/pkg/kafka"
	"slate/services/assignment-grading-service/pkg/logger"
	"slate/services/assignment-grading-service/pkg/metrics"
	"slate/services/assignment-grading-service/pkg/storage"
	"slate/services/assignment-grading-service/pkg/tracing"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
	"google.golang.org/grpc"
	"google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/reflection"
)

func main() {
	// Initialize logger
	logLevel := os.Getenv("LOG_LEVEL")
	if logLevel == "" {
		logLevel = "info"
	}
	log := logger.NewLogger(logLevel)
	log.Info().Str("log_level", logLevel).Msg("Starting Assignment Grading Service")

	// Load configuration
	cfg, err := config.Load()
	if err != nil {
		log.Error().Err(err).Msg("Failed to load configuration")
		os.Exit(1)
	}

	// Initialize OpenTelemetry tracing
	tracingCfg := tracing.Config{
		ServiceName:    "assignment-grading-service",
		ServiceVersion: "1.0.0",
		OTLPEndpoint:   cfg.Observability.OTLPEndpoint,
		OTLPInsecure:   cfg.Observability.OTLPInsecure,
		SamplingRate:   1.0,
	}

	log.Info().Str("otlp_endpoint", cfg.Observability.OTLPEndpoint).Msg("Initializing OpenTelemetry tracing")
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
		if tp != nil {
			shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			if shutdownErr := tracing.Shutdown(shutdownCtx, tp); shutdownErr != nil {
				log.Error().Err(shutdownErr).Msg("Failed to shutdown tracer")
			}
			cancel()
		}
		os.Exit(1)
	}
	defer db.Close()

	log.Info().Msg("Connected to PostgreSQL database")

	// Run migrations
	migrationsPath := filepath.Join(".", "migrations")
	if _, err := os.Stat(migrationsPath); os.IsNotExist(err) {
		migrationsPath = "/app/migrations"
	}

	if err := migrations.RunMigrations(db.DB, migrationsPath); err != nil {
		log.Error().Err(err).Msg("Failed to run migrations")
		os.Exit(1)
	}

	// Initialize Prometheus metrics
	registry := prometheus.NewRegistry()
	metricsCollector := metrics.NewMetrics(registry)
	log.Info().Msg("Prometheus metrics initialized")

	// Start metrics HTTP server
	metricsAddr := fmt.Sprintf(":%d", cfg.Observability.MetricsPort)
	metricsServer := &http.Server{
		Addr:              metricsAddr,
		Handler:           promhttp.HandlerFor(registry, promhttp.HandlerOpts{Registry: registry}),
		ReadHeaderTimeout: 10 * time.Second,
	}

	go func() {
		log.Info().Str("address", metricsAddr).Msg("Starting metrics HTTP server")
		if err := metricsServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Error().Err(err).Msg("Metrics server failed")
		}
	}()

	// Initialize Kafka producer
	kafkaProducer := kafka.NewProducer(cfg.Kafka.Brokers, cfg.Kafka.Topic, cfg.Kafka.Enabled)
	if cfg.Kafka.Enabled {
		log.Info().Strs("brokers", cfg.Kafka.Brokers).Msg("Kafka producer initialized")
		defer kafkaProducer.Close()
	} else {
		log.Info().Msg("Kafka is disabled")
	}

	// Initialize file storage
	fileStorage, err := storage.NewLocalFileStorage(cfg.Storage.LocalPath, cfg.Storage.MaxSize)
	if err != nil {
		log.Error().Err(err).Msg("Failed to initialize file storage")
		os.Exit(1)
	}
	log.Info().Str("path", cfg.Storage.LocalPath).Msg("File storage initialized")

	// Initialize repositories
	assignmentRepo := repository.NewAssignmentRepository(db.DB)
	submissionRepo := repository.NewSubmissionRepository(db.DB)
	gradeRepo := repository.NewGradeRepository(db.DB)

	// Initialize services
	assignmentService := service.NewAssignmentService(assignmentRepo, kafkaProducer)
	submissionService := service.NewSubmissionService(assignmentRepo, submissionRepo, fileStorage, kafkaProducer)
	gradingService := service.NewGradingService(assignmentRepo, submissionRepo, gradeRepo, kafkaProducer)
	gradebookService := service.NewGradebookService(assignmentRepo, submissionRepo, gradeRepo)

	log.Info().Msg("Services initialized")

	// TODO: Initialize gRPC handlers when proto code is generated
	// For now, create a basic gRPC server
	_ = assignmentService
	_ = submissionService
	_ = gradingService
	_ = gradebookService
	_ = metricsCollector

	// Initialize gRPC server with OpenTelemetry interceptors
	grpcServer := grpc.NewServer(
		grpc.StatsHandler(otelgrpc.NewServerHandler()),
		grpc.ChainUnaryInterceptor(
			tracing.LoggingUnaryInterceptor(),
		),
	)

	// Register health check service
	healthChecker := health.NewHealthChecker(db.DB)
	grpc_health_v1.RegisterHealthServer(grpcServer, healthChecker)
	log.Info().Msg("gRPC health check service registered")

	// Enable reflection for debugging
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
