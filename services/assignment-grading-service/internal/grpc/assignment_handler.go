package grpc

import (
	"context"

	pb "slate/services/assignment-grading-service/api/proto"
	"slate/services/assignment-grading-service/internal/service"

	"google.golang.org/protobuf/types/known/timestamppb"
)

// AssignmentServiceServer implements the AssignmentService gRPC interface
type AssignmentServiceServer struct {
	pb.UnimplementedAssignmentServiceServer
	assignmentService *service.AssignmentService
}

// NewAssignmentServiceServer creates a new AssignmentServiceServer
func NewAssignmentServiceServer(assignmentService *service.AssignmentService) *AssignmentServiceServer {
	return &AssignmentServiceServer{
		assignmentService: assignmentService,
	}
}

// Ping tests connectivity to the service
func (s *AssignmentServiceServer) Ping(ctx context.Context, req *pb.PingRequest) (*pb.PingResponse, error) {
	message := req.Message
	if message == "" {
		message = "pong"
	}

	return &pb.PingResponse{
		Message:   message,
		Service:   "assignment-grading-service",
		Timestamp: timestamppb.Now(),
	}, nil
}
