package grpc

import (
	"strings"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// mapError converts service layer errors to appropriate gRPC status codes
func mapError(err error) error {
	if err == nil {
		return nil
	}

	errMsg := err.Error()

	// Not found errors
	if strings.Contains(errMsg, "not found") {
		return status.Error(codes.NotFound, errMsg)
	}

	// Validation errors
	if strings.Contains(errMsg, "validation failed") ||
		strings.Contains(errMsg, "is required") ||
		strings.Contains(errMsg, "must be") ||
		strings.Contains(errMsg, "invalid") ||
		strings.Contains(errMsg, "exceeds") {
		return status.Error(codes.InvalidArgument, errMsg)
	}

	// Permission/precondition errors
	if strings.Contains(errMsg, "cannot update") ||
		strings.Contains(errMsg, "cannot delete") ||
		strings.Contains(errMsg, "already published") ||
		strings.Contains(errMsg, "with existing submissions") {
		return status.Error(codes.FailedPrecondition, errMsg)
	}

	// Default to internal error
	return status.Error(codes.Internal, "internal server error")
}
