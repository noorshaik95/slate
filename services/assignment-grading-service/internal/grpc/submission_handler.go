package grpc

import (
	"context"
	pb "slate/services/assignment-grading-service/api/proto"
	"slate/services/assignment-grading-service/internal/service"
)

// SubmissionServiceServer implements the SubmissionService gRPC interface
type SubmissionServiceServer struct {
	pb.UnimplementedSubmissionServiceServer
	service service.SubmissionService
}

// NewSubmissionServiceServer creates a new SubmissionServiceServer
func NewSubmissionServiceServer(svc service.SubmissionService) *SubmissionServiceServer {
	return &SubmissionServiceServer{
		service: svc,
	}
}

// SubmitAssignment accepts a student submission with file content
func (s *SubmissionServiceServer) SubmitAssignment(ctx context.Context, req *pb.SubmitAssignmentRequest) (*pb.SubmitAssignmentResponse, error) {
	log.WithContext(ctx).
		Str("assignment_id", req.AssignmentId).
		Str("student_id", req.StudentId).
		Str("file_name", req.FileName).
		Msg("SubmitAssignment called")

	// Call service layer with file content
	submission, err := s.service.SubmitAssignment(
		ctx,
		req.AssignmentId,
		req.StudentId,
		req.FileContent,
		req.FileName,
		req.ContentType,
	)

	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("assignment_id", req.AssignmentId).
			Str("student_id", req.StudentId).
			Msg("Failed to submit assignment")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("submission_id", submission.ID).
		Str("assignment_id", req.AssignmentId).
		Str("student_id", req.StudentId).
		Bool("is_late", submission.IsLate).
		Msg("Assignment submitted successfully")

	return &pb.SubmitAssignmentResponse{
		Submission: submissionToProto(submission),
	}, nil
}

// GetSubmission retrieves a submission by ID
func (s *SubmissionServiceServer) GetSubmission(ctx context.Context, req *pb.GetSubmissionRequest) (*pb.GetSubmissionResponse, error) {
	log.WithContext(ctx).
		Str("submission_id", req.Id).
		Msg("GetSubmission called")

	submission, err := s.service.GetSubmission(ctx, req.Id)
	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("submission_id", req.Id).
			Msg("Failed to get submission")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("submission_id", submission.ID).
		Msg("Submission retrieved successfully")

	return &pb.GetSubmissionResponse{
		Submission: submissionToProto(submission),
	}, nil
}

// ListSubmissions lists all submissions for an assignment with sorting
func (s *SubmissionServiceServer) ListSubmissions(ctx context.Context, req *pb.ListSubmissionsRequest) (*pb.ListSubmissionsResponse, error) {
	log.WithContext(ctx).
		Str("assignment_id", req.AssignmentId).
		Str("sort_by", req.SortBy).
		Str("order", req.Order).
		Msg("ListSubmissions called")

	submissions, err := s.service.ListSubmissions(
		ctx,
		req.AssignmentId,
		req.SortBy,
		req.Order,
	)

	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("assignment_id", req.AssignmentId).
			Msg("Failed to list submissions")
		return nil, mapError(err)
	}

	// Convert submissions to proto
	protoSubmissions := make([]*pb.Submission, 0, len(submissions))
	for _, submission := range submissions {
		protoSubmissions = append(protoSubmissions, submissionToProto(submission))
	}

	log.WithContext(ctx).
		Str("assignment_id", req.AssignmentId).
		Int("count", len(submissions)).
		Msg("Submissions listed successfully")

	return &pb.ListSubmissionsResponse{
		Submissions: protoSubmissions,
	}, nil
}

// ListStudentSubmissions lists all submissions by a student for a course
func (s *SubmissionServiceServer) ListStudentSubmissions(ctx context.Context, req *pb.ListStudentSubmissionsRequest) (*pb.ListStudentSubmissionsResponse, error) {
	log.WithContext(ctx).
		Str("student_id", req.StudentId).
		Str("course_id", req.CourseId).
		Msg("ListStudentSubmissions called")

	submissions, err := s.service.ListStudentSubmissions(
		ctx,
		req.StudentId,
		req.CourseId,
	)

	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("student_id", req.StudentId).
			Str("course_id", req.CourseId).
			Msg("Failed to list student submissions")
		return nil, mapError(err)
	}

	// Convert submissions to proto
	protoSubmissions := make([]*pb.Submission, 0, len(submissions))
	for _, submission := range submissions {
		protoSubmissions = append(protoSubmissions, submissionToProto(submission))
	}

	log.WithContext(ctx).
		Str("student_id", req.StudentId).
		Str("course_id", req.CourseId).
		Int("count", len(submissions)).
		Msg("Student submissions listed successfully")

	return &pb.ListStudentSubmissionsResponse{
		Submissions: protoSubmissions,
	}, nil
}
