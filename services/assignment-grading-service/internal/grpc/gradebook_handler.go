package grpc

import (
	"context"
	pb "slate/services/assignment-grading-service/api/proto"
	"slate/services/assignment-grading-service/internal/service"
)

// GradebookServiceServer implements the GradebookService gRPC interface
type GradebookServiceServer struct {
	pb.UnimplementedGradebookServiceServer
	service service.GradebookService
}

// NewGradebookServiceServer creates a new GradebookServiceServer
func NewGradebookServiceServer(svc service.GradebookService) *GradebookServiceServer {
	return &GradebookServiceServer{
		service: svc,
	}
}

// GetStudentGradebook returns all grades for a student in a course
func (s *GradebookServiceServer) GetStudentGradebook(ctx context.Context, req *pb.GetStudentGradebookRequest) (*pb.GetStudentGradebookResponse, error) {
	log.WithContext(ctx).
		Str("student_id", req.StudentId).
		Str("course_id", req.CourseId).
		Msg("GetStudentGradebook called")

	// Call service layer (computes totals, percentages, letter grades)
	gradebook, err := s.service.GetStudentGradebook(ctx, req.StudentId, req.CourseId)
	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("student_id", req.StudentId).
			Str("course_id", req.CourseId).
			Msg("Failed to get student gradebook")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("student_id", req.StudentId).
		Str("course_id", req.CourseId).
		Int("entries", len(gradebook.Entries)).
		Float64("percentage", gradebook.Percentage).
		Str("letter_grade", gradebook.LetterGrade).
		Msg("Student gradebook retrieved successfully")

	return studentGradebookToProto(gradebook), nil
}

// GetCourseGradebook returns grades for all students in a course
func (s *GradebookServiceServer) GetCourseGradebook(ctx context.Context, req *pb.GetCourseGradebookRequest) (*pb.GetCourseGradebookResponse, error) {
	log.WithContext(ctx).
		Str("course_id", req.CourseId).
		Msg("GetCourseGradebook called")

	// Call service layer (aggregates all student grades)
	gradebook, err := s.service.GetCourseGradebook(ctx, req.CourseId)
	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("course_id", req.CourseId).
			Msg("Failed to get course gradebook")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("course_id", req.CourseId).
		Int("students", len(gradebook.Students)).
		Msg("Course gradebook retrieved successfully")

	return courseGradebookToProto(gradebook), nil
}

// GetGradeStatistics calculates statistical metrics for an assignment
func (s *GradebookServiceServer) GetGradeStatistics(ctx context.Context, req *pb.GetGradeStatisticsRequest) (*pb.GetGradeStatisticsResponse, error) {
	log.WithContext(ctx).
		Str("assignment_id", req.AssignmentId).
		Msg("GetGradeStatistics called")

	// Call service layer (computes mean, median, std deviation, min, max)
	statistics, err := s.service.GetGradeStatistics(ctx, req.AssignmentId)
	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("assignment_id", req.AssignmentId).
			Msg("Failed to get grade statistics")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("assignment_id", req.AssignmentId).
		Int("total_submissions", statistics.TotalSubmissions).
		Int("graded_count", statistics.GradedCount).
		Float64("mean", statistics.Mean).
		Msg("Grade statistics retrieved successfully")

	return &pb.GetGradeStatisticsResponse{
		Statistics: gradeStatisticsToProto(statistics, req.AssignmentId),
	}, nil
}

// ExportGrades generates CSV or JSON export of course grades
func (s *GradebookServiceServer) ExportGrades(ctx context.Context, req *pb.ExportGradesRequest) (*pb.ExportGradesResponse, error) {
	log.WithContext(ctx).
		Str("course_id", req.CourseId).
		Str("format", req.Format).
		Msg("ExportGrades called")

	// Call service layer (generates export in requested format)
	data, err := s.service.ExportGrades(ctx, req.CourseId, req.Format)
	if err != nil {
		log.ErrorWithContext(ctx).
			Err(err).
			Str("course_id", req.CourseId).
			Str("format", req.Format).
			Msg("Failed to export grades")
		return nil, mapError(err)
	}

	log.WithContext(ctx).
		Str("course_id", req.CourseId).
		Str("format", req.Format).
		Int("size_bytes", len(data)).
		Msg("Grades exported successfully")

	return &pb.ExportGradesResponse{
		Data:   data,
		Format: req.Format,
	}, nil
}
