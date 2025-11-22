package grpc

import (
	"context"
	pb "slate/services/assignment-grading-service/api/proto"
	"slate/services/assignment-grading-service/internal/service"
)

// GradingServiceServer implements the GradingService gRPC interface
type GradingServiceServer struct {
	pb.UnimplementedGradingServiceServer
	service service.GradingService
}

// NewGradingServiceServer creates a new GradingServiceServer
func NewGradingServiceServer(svc service.GradingService) *GradingServiceServer {
	return &GradingServiceServer{
		service: svc,
	}
}

// CreateGrade creates a draft grade with score and feedback
func (s *GradingServiceServer) CreateGrade(ctx context.Context, req *pb.CreateGradeRequest) (*pb.CreateGradeResponse, error) {
	log.WithContext(ctx).
		Str("submission_id", req.SubmissionId).
		Float64("score", req.Score).
		Str("graded_by", req.GradedBy).
		Msg("CreateGrade called")

	// Call service layer (handles adjusted score calculation with late penalties)
	grade, err := s.service.CreateGrade(
		ctx,
		req.SubmissionId,
		req.Score,
		req.Feedback,
		req.GradedBy,
	)

	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("submission_id", req.SubmissionId).
			Msg("Failed to create grade")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("grade_id", grade.ID).
		Str("submission_id", req.SubmissionId).
		Float64("score", grade.Score).
		Float64("adjusted_score", grade.AdjustedScore).
		Msg("Grade created successfully")

	return &pb.CreateGradeResponse{
		Grade: gradeToProto(grade),
	}, nil
}

// UpdateGrade updates a draft grade's score and feedback
func (s *GradingServiceServer) UpdateGrade(ctx context.Context, req *pb.UpdateGradeRequest) (*pb.UpdateGradeResponse, error) {
	log.WithContext(ctx).
		Str("grade_id", req.Id).
		Float64("score", req.Score).
		Msg("UpdateGrade called")

	// Call service layer (checks draft status and recalculates adjusted score)
	grade, err := s.service.UpdateGrade(
		ctx,
		req.Id,
		req.Score,
		req.Feedback,
	)

	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("grade_id", req.Id).
			Msg("Failed to update grade")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("grade_id", grade.ID).
		Float64("score", grade.Score).
		Float64("adjusted_score", grade.AdjustedScore).
		Msg("Grade updated successfully")

	return &pb.UpdateGradeResponse{
		Grade: gradeToProto(grade),
	}, nil
}

// PublishGrade publishes a grade to make it visible to students
func (s *GradingServiceServer) PublishGrade(ctx context.Context, req *pb.PublishGradeRequest) (*pb.PublishGradeResponse, error) {
	log.WithContext(ctx).
		Str("grade_id", req.Id).
		Msg("PublishGrade called")

	// Call service layer (updates status and published_at timestamp)
	grade, err := s.service.PublishGrade(ctx, req.Id)
	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("grade_id", req.Id).
			Msg("Failed to publish grade")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("grade_id", grade.ID).
		Str("status", grade.Status).
		Msg("Grade published successfully")

	return &pb.PublishGradeResponse{
		Grade: gradeToProto(grade),
	}, nil
}

// GetGrade retrieves a grade by ID
func (s *GradingServiceServer) GetGrade(ctx context.Context, req *pb.GetGradeRequest) (*pb.GetGradeResponse, error) {
	log.WithContext(ctx).
		Str("grade_id", req.Id).
		Msg("GetGrade called")

	grade, err := s.service.GetGrade(ctx, req.Id)
	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("grade_id", req.Id).
			Msg("Failed to get grade")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("grade_id", grade.ID).
		Msg("Grade retrieved successfully")

	return &pb.GetGradeResponse{
		Grade: gradeToProto(grade),
	}, nil
}
