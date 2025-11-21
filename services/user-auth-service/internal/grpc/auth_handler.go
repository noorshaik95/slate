package grpc

import (
	"context"
	"time"

	pb "slate/services/user-auth-service/api/proto"
	authpb "slate/services/user-auth-service/api/proto/authpb"
)

// AuthServiceServer implements the auth.AuthService gRPC interface
// by delegating to the existing UserServiceServer implementation.
// This provides a thin wrapper that allows the API Gateway to call
// auth.AuthService/ValidateToken while reusing all existing validation logic.
type AuthServiceServer struct {
	authpb.UnimplementedAuthServiceServer
	userServiceServer *UserServiceServer
}

// NewAuthServiceServer creates a new AuthServiceServer that wraps
// the existing UserServiceServer for token validation.
func NewAuthServiceServer(userServiceServer *UserServiceServer) *AuthServiceServer {
	return &AuthServiceServer{
		userServiceServer: userServiceServer,
	}
}

// ValidateToken validates a JWT token by delegating to UserServiceServer.
// This method converts between auth.proto and user.proto message types,
// which have identical fields, making the conversion trivial.
func (s *AuthServiceServer) ValidateToken(ctx context.Context, req *authpb.ValidateTokenRequest) (*authpb.ValidateTokenResponse, error) {
	// Convert auth.ValidateTokenRequest to user.ValidateTokenRequest
	userReq := &pb.ValidateTokenRequest{
		Token: req.Token,
	}

	// Delegate to existing UserServiceServer.ValidateToken
	userResp, err := s.userServiceServer.ValidateToken(ctx, userReq)
	if err != nil {
		return nil, err
	}

	// Convert user.ValidateTokenResponse to auth.ValidateTokenResponse
	// Both proto messages have identical fields: valid, user_id, roles, error
	authResp := &authpb.ValidateTokenResponse{
		Valid:  userResp.Valid,
		UserId: userResp.UserId,
		Roles:  userResp.Roles,
		Error:  userResp.Error,
	}

	return authResp, nil
}

// HealthCheck returns the health status of the auth service
func (s *AuthServiceServer) HealthCheck(ctx context.Context, req *authpb.HealthCheckRequest) (*authpb.HealthCheckResponse, error) {
	return &authpb.HealthCheckResponse{
		Status:    "healthy",
		Service:   "user-auth-service",
		Timestamp: time.Now().Format(time.RFC3339),
	}, nil
}

