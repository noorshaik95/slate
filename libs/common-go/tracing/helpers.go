// Package tracing provides reusable OpenTelemetry tracing utilities for Go services.
// It offers helper functions to reduce boilerplate when creating and managing spans.
package tracing

import (
	"context"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
)

// StartSpan creates a new span with the given name and attributes.
// It returns the updated context and the created span.
// The span should be ended using defer span.End() or EndSpanWithError.
//
// Usage:
//
//	ctx, span := tracing.StartSpan(ctx, "operation_name",
//	    attribute.String("key", "value"))
//	defer span.End()
func StartSpan(ctx context.Context, spanName string, attrs ...attribute.KeyValue) (context.Context, trace.Span) {
	// Extract tracer name from context or use default
	tracerName := getTracerName(ctx)
	tracer := otel.Tracer(tracerName)

	ctx, span := tracer.Start(ctx, spanName)

	if len(attrs) > 0 {
		span.SetAttributes(attrs...)
	}

	return ctx, span
}

// StartSpanWithTracer creates a new span using a specific tracer name.
// Useful when you want to explicitly set the service name.
//
// Usage:
//
//	ctx, span := tracing.StartSpanWithTracer(ctx, "user-auth-service", "operation_name",
//	    attribute.String("key", "value"))
//	defer span.End()
func StartSpanWithTracer(ctx context.Context, tracerName, spanName string, attrs ...attribute.KeyValue) (context.Context, trace.Span) {
	tracer := otel.Tracer(tracerName)
	ctx, span := tracer.Start(ctx, spanName)

	if len(attrs) > 0 {
		span.SetAttributes(attrs...)
	}

	return ctx, span
}

// EndSpanWithError ends a span and records an error if present.
// It sets the span status based on whether an error occurred.
//
// Usage:
//
//	func myFunc(ctx context.Context) (err error) {
//	    ctx, span := tracing.StartSpan(ctx, "my_func")
//	    defer tracing.EndSpanWithError(span, &err)
//	    // ... function logic
//	}
func EndSpanWithError(span trace.Span, err *error) {
	if err != nil && *err != nil {
		span.RecordError(*err)
		span.SetStatus(codes.Error, (*err).Error())
	} else {
		span.SetStatus(codes.Ok, "")
	}
	span.End()
}

// AddSpanAttributes adds attributes to an existing span.
// Useful for adding context discovered during execution.
//
// Usage:
//
//	tracing.AddSpanAttributes(span,
//	    attribute.String("user_id", userID),
//	    attribute.Int("count", count))
func AddSpanAttributes(span trace.Span, attrs ...attribute.KeyValue) {
	if len(attrs) > 0 {
		span.SetAttributes(attrs...)
	}
}

// RecordError records an error on the span without ending it.
// Useful when you want to record an error but continue processing.
//
// Usage:
//
//	if err := step1(); err != nil {
//	    tracing.RecordError(span, err, "step1 failed")
//	    // continue processing or return
//	}
func RecordError(span trace.Span, err error, description string) {
	if err != nil {
		span.RecordError(err)
		if description != "" {
			span.SetStatus(codes.Error, description)
		} else {
			span.SetStatus(codes.Error, err.Error())
		}
	}
}

// SetSpanStatus explicitly sets the span status.
//
// Usage:
//
//	tracing.SetSpanStatus(span, codes.Ok, "operation completed")
func SetSpanStatus(span trace.Span, code codes.Code, description string) {
	span.SetStatus(code, description)
}

// tracerNameKey is the context key for storing tracer name
type tracerNameKey struct{}

// WithTracerName adds a tracer name to the context.
// This allows StartSpan to use the correct service name.
//
// Usage:
//
//	ctx = tracing.WithTracerName(ctx, "user-auth-service")
func WithTracerName(ctx context.Context, name string) context.Context {
	return context.WithValue(ctx, tracerNameKey{}, name)
}

// getTracerName extracts the tracer name from context or returns default.
func getTracerName(ctx context.Context) string {
	if name, ok := ctx.Value(tracerNameKey{}).(string); ok {
		return name
	}
	return "default-service"
}
