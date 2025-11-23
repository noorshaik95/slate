// Package tracing provides reusable OpenTelemetry tracing utilities for Go services.
//
// This package offers helper functions to simplify the creation and management of
// distributed tracing spans across microservices. It reduces boilerplate code and
// ensures consistent tracing patterns.
//
// # Basic Usage
//
// Import the package:
//
//	import "slate/libs/common-go/tracing"
//
// Create a span with defer pattern:
//
//	func MyHandler(ctx context.Context) error {
//	    ctx, span := tracing.StartSpan(ctx, "my_handler",
//	        attribute.String("key", "value"))
//	    defer span.End()
//
//	    // Your handler logic
//	    return nil
//	}
//
// # Error Handling
//
// Use EndSpanWithError for automatic error recording:
//
//	func MyService(ctx context.Context) (err error) {
//	    ctx, span := tracing.StartSpan(ctx, "my_service")
//	    defer tracing.EndSpanWithError(span, &err)
//
//	    // Your service logic that may return err
//	    return someOperation()
//	}
//
// # Setting Tracer Name
//
// Set the service name for proper trace attribution:
//
//	func main() {
//	    ctx := context.Background()
//	    ctx = tracing.WithTracerName(ctx, "user-auth-service")
//
//	    // Pass ctx to handlers
//	}
package tracing
