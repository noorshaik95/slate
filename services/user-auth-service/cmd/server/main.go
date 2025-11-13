package main

import (
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"

	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/config"
	grpcHandler "github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/grpc"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/repository"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/service"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/migrations"
	pb "github.com/noorshaik95/axum-grafana-example/services/user-auth-service/api/proto"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/pkg/database"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/pkg/jwt"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/pkg/logger"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

func main() {
	// Initialize logger
	log := logger.New()
	log.Info("Starting User Auth Service...")

	// Load configuration
	cfg, err := config.Load()
	if err != nil {
		log.Error("Failed to load configuration:", err)
		os.Exit(1)
	}

	// Connect to database
	db, err := database.NewPostgresDB(cfg.Database.DSN())
	if err != nil {
		log.Error("Failed to connect to database:", err)
		os.Exit(1)
	}
	defer db.Close()

	log.Info("Connected to PostgreSQL database")

	// Run migrations
	migrationsPath := filepath.Join(".", "migrations")
	if _, err := os.Stat(migrationsPath); os.IsNotExist(err) {
		// Try alternative path for Docker
		migrationsPath = "/app/migrations"
	}

	if err := migrations.RunMigrations(db.DB, migrationsPath); err != nil {
		log.Error("Failed to run migrations:", err)
		os.Exit(1)
	}

	// Initialize repositories
	userRepo := repository.NewUserRepository(db.DB)
	roleRepo := repository.NewRoleRepository(db.DB)

	// Ensure default roles exist
	if err := roleRepo.EnsureDefaultRoles(); err != nil {
		log.Error("Failed to ensure default roles:", err)
		os.Exit(1)
	}

	// Initialize JWT service
	tokenService := jwt.NewTokenService(
		cfg.JWT.SecretKey,
		cfg.JWT.AccessTokenDuration,
		cfg.JWT.RefreshTokenDuration,
	)

	// Initialize services
	userService := service.NewUserService(userRepo, roleRepo, tokenService)

	// Initialize gRPC server
	grpcServer := grpc.NewServer()

	// Register services
	userServiceServer := grpcHandler.NewUserServiceServer(userService)
	pb.RegisterUserServiceServer(grpcServer, userServiceServer)

	// Enable reflection for debugging with tools like grpcurl
	reflection.Register(grpcServer)

	// Start gRPC server
	lis, err := net.Listen("tcp", cfg.GRPC.Address())
	if err != nil {
		log.Error("Failed to listen on", cfg.GRPC.Address(), ":", err)
		os.Exit(1)
	}

	log.Info("gRPC server listening on", cfg.GRPC.Address())

	// Handle graceful shutdown
	go func() {
		sigint := make(chan os.Signal, 1)
		signal.Notify(sigint, os.Interrupt, syscall.SIGTERM)
		<-sigint

		log.Info("Shutting down gRPC server...")
		grpcServer.GracefulStop()
	}()

	// Start serving
	if err := grpcServer.Serve(lis); err != nil {
		log.Error("Failed to serve:", err)
		os.Exit(1)
	}

	log.Info("Server stopped")
}
