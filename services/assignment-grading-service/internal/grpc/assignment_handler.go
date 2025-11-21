package grpc

import (
	"context"
	"time"

	pb "slate/services/assignment-grading-service/api/proto"
	"slate/services/assignment-grading-service/internal/service"
	"slate/services/assignment-grading-service/pkg/logger"
)

var log = logger.NewLogger("info")

// AssignmentServiceServer implements the AssignmentService gRPC interface
type AssignmentServiceServer struct {
	pb.UnimplementedAssignmentServiceServer
	service service.AssignmentService
}

// NewAssignmentServiceServer creates a new AssignmentServiceServer
func NewAssignmentServiceServer(svc service.AssignmentService) *AssignmentServiceServer {
	return &AssignmentServiceServer{
		service: svc,
	}
}

// HealthCheck returns the health status of the assignment grading service
func (s *AssignmentServiceServer) HealthCheck(ctx context.Context, req *pb.HealthCheckRequest) (*pb.HealthCheckResponse, error) {
	return &pb.HealthCheckResponse{
		Status:    "healthy",
		Service:   "assignment-grading-service",
		Timestamp: time.Now().Format(time.RFC3339),
	}, nil
}

// CreateAssignment creates a new assignment
func (s *AssignmentServiceServer) CreateAssignment(ctx context.Context, req *pb.CreateAssignmentRequest) (*pb.CreateAssignmentResponse, error) {
	log.Info().
		Str("course_id", req.CourseId).
		Str("title", req.Title).
		Msg("CreateAssignment called")

	// Convert proto late policy to model
	latePolicy := protoToLatePolicy(req.LatePolicy)

	// Call service layer
	assignment, err := s.service.CreateAssignment(
		ctx,
		req.CourseId,
		req.Title,
		req.Description,
		req.MaxPoints,
		req.DueDate.AsTime(),
		latePolicy,
	)

	if err != nil {
		log.Error().
			Err(err).
			Str("course_id", req.CourseId).
			Msg("Failed to create assignment")
		return nil, mapError(err)
	}

	log.Info().
		Str("assignment_id", assignment.ID).
		Msg("Assignment created successfully")

	return &pb.CreateAssignmentResponse{
		Assignment: assignmentToProto(assignment),
	}, nil
}

// GetAssignment retrieves an assignment by ID
func (s *AssignmentServiceServer) GetAssignment(ctx context.Context, req *pb.GetAssignmentRequest) (*pb.GetAssignmentResponse, error) {
	log.Info().
		Str("assignment_id", req.Id).
		Msg("GetAssignment called")

	assignment, err := s.service.GetAssignment(ctx, req.Id)
	if err != nil {
		log.Error().
			Err(err).
			Str("assignment_id", req.Id).
			Msg("Failed to get assignment")
		return nil, mapError(err)
	}

	log.Info().
		Str("assignment_id", assignment.ID).
		Msg("Assignment retrieved successfully")

	return &pb.GetAssignmentResponse{
		Assignment: assignmentToProto(assignment),
	}, nil
}

// UpdateAssignment updates an existing assignment
func (s *AssignmentServiceServer) UpdateAssignment(ctx context.Context, req *pb.UpdateAssignmentRequest) (*pb.UpdateAssignmentResponse, error) {
	log.Info().
		Str("assignment_id", req.Id).
		Str("title", req.Title).
		Msg("UpdateAssignment called")

	// Convert proto late policy to model
	latePolicy := protoToLatePolicy(req.LatePolicy)

	// Call service layer
	assignment, err := s.service.UpdateAssignment(
		ctx,
		req.Id,
		req.Title,
		req.Description,
		req.MaxPoints,
		req.DueDate.AsTime(),
		latePolicy,
	)

	if err != nil {
		log.Error().
			Err(err).
			Str("assignment_id", req.Id).
			Msg("Failed to update assignment")
		return nil, mapError(err)
	}

	log.Info().
		Str("assignment_id", assignment.ID).
		Msg("Assignment updated successfully")

	return &pb.UpdateAssignmentResponse{
		Assignment: assignmentToProto(assignment),
	}, nil
}

// DeleteAssignment deletes an assignment
func (s *AssignmentServiceServer) DeleteAssignment(ctx context.Context, req *pb.DeleteAssignmentRequest) (*pb.DeleteAssignmentResponse, error) {
	log.Info().
		Str("assignment_id", req.Id).
		Msg("DeleteAssignment called")

	err := s.service.DeleteAssignment(ctx, req.Id)
	if err != nil {
		log.Error().
			Err(err).
			Str("assignment_id", req.Id).
			Msg("Failed to delete assignment")
		return nil, mapError(err)
	}

	log.Info().
		Str("assignment_id", req.Id).
		Msg("Assignment deleted successfully")

	return &pb.DeleteAssignmentResponse{
		Success: true,
	}, nil
}

// ListAssignments lists assignments for a course with pagination
func (s *AssignmentServiceServer) ListAssignments(ctx context.Context, req *pb.ListAssignmentsRequest) (*pb.ListAssignmentsResponse, error) {
	log.Info().
		Str("course_id", req.CourseId).
		Int32("page", req.Page).
		Int32("page_size", req.PageSize).
		Msg("ListAssignments called")

	// Call service layer
	assignments, total, err := s.service.ListAssignments(
		ctx,
		req.CourseId,
		int(req.Page),
		int(req.PageSize),
	)

	if err != nil {
		log.Error().
			Err(err).
			Str("course_id", req.CourseId).
			Msg("Failed to list assignments")
		return nil, mapError(err)
	}

	// Convert assignments to proto
	protoAssignments := make([]*pb.Assignment, 0, len(assignments))
	for _, assignment := range assignments {
		protoAssignments = append(protoAssignments, assignmentToProto(assignment))
	}

	log.Info().
		Str("course_id", req.CourseId).
		Int("count", len(assignments)).
		Int("total", total).
		Msg("Assignments listed successfully")

	return &pb.ListAssignmentsResponse{
		Assignments: protoAssignments,
		Total:       int32(total),
		Page:        req.Page,
		PageSize:    req.PageSize,
	}, nil
}
