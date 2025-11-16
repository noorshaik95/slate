package tracing

import (
	"context"
	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	semconv "go.opentelemetry.io/otel/semconv/v1.17.0"
	"go.opentelemetry.io/otel/trace"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"
)

// LoggingUnaryInterceptor logs incoming gRPC metadata and trace context for debugging
func LoggingUnaryInterceptor() grpc.UnaryServerInterceptor {
	log := logger.NewLogger("debug")

	return func(
		ctx context.Context,
		req any,
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (any, error) {
		// Extract metadata from context
		md, ok := metadata.FromIncomingContext(ctx)
		if ok {
			log.Debug().
				Str("method", info.FullMethod).
				Int("metadata_keys", len(md)).
				Msg("gRPC interceptor called")

			// Log trace-related headers
			if traceparent := md.Get("traceparent"); len(traceparent) > 0 {
				log.Debug().
					Str("traceparent", traceparent[0]).
					Msg("Traceparent header found")
			} else {
				log.Warn().
					Str("method", info.FullMethod).
					Msg("No traceparent header in request")
			}

			if tracestate := md.Get("tracestate"); len(tracestate) > 0 {
				log.Debug().
					Str("tracestate", tracestate[0]).
					Msg("Tracestate header found")
			}
		} else {
			log.Warn().
				Str("method", info.FullMethod).
				Msg("No metadata in gRPC context")
		}

		// Check if there's already a span in the context (from otelgrpc)
		span := trace.SpanFromContext(ctx)
		if span.SpanContext().IsValid() {
			log.Debug().
				Str("trace_id", span.SpanContext().TraceID().String()).
				Str("span_id", span.SpanContext().SpanID().String()).
				Msg("Valid span found in context")
		} else {
			log.Warn().
				Str("method", info.FullMethod).
				Msg("No valid span in context")
		}

		// Create a new span for this operation
		tracer := otel.Tracer("user-auth-service")
		ctx, span = tracer.Start(ctx, "grpc."+info.FullMethod)
		defer span.End()

		log.Debug().
			Str("span_id", span.SpanContext().SpanID().String()).
			Msg("Created new span for gRPC method")

		// Call the actual handler
		return handler(ctx, req)
	}
}

// metadataCarrier adapts gRPC metadata to be used as a TextMapCarrier for trace context propagation
type metadataCarrier struct {
	md metadata.MD
}

// Get retrieves a value from the metadata by key
func (mc metadataCarrier) Get(key string) string {
	values := mc.md.Get(key)
	if len(values) == 0 {
		return ""
	}
	return values[0]
}

// Set sets a value in the metadata (not used for extraction, only injection)
func (mc metadataCarrier) Set(key, value string) {
	mc.md.Set(key, value)
}

// Keys returns all keys in the metadata
func (mc metadataCarrier) Keys() []string {
	keys := make([]string, 0, len(mc.md))
	for k := range mc.md {
		keys = append(keys, k)
	}
	return keys
}

// TracingUnaryInterceptor creates a gRPC unary interceptor that extracts trace context
// from incoming requests and creates spans for each RPC call. This interceptor:
//   - Extracts trace context from gRPC metadata using W3C Trace Context propagation
//   - Creates a span from the extracted context or a new root span if no context exists
//   - Sets span attributes including RPC system, service, method, and peer IP
//   - Injects the span context into the request context for downstream use
//   - Records the response status in the span
//   - Properly handles errors by recording them in the span
//
// Note: This interceptor works alongside otelgrpc.NewServerHandler() which provides
// automatic instrumentation. This interceptor is useful for explicit control over
// span attributes and for debugging trace propagation.
func TracingUnaryInterceptor() grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req any,
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (any, error) {
		// Extract metadata from incoming context
		md, ok := metadata.FromIncomingContext(ctx)
		if !ok {
			md = metadata.New(nil)
		}

		// Extract trace context from metadata using the global propagator
		propagator := otel.GetTextMapPropagator()
		carrier := metadataCarrier{md: md}
		extractedCtx := propagator.Extract(ctx, carrier)

		// Create tracer
		tracer := otel.Tracer("user-auth-service")

		// Start a new span from the extracted context
		// If no parent context exists, this will create a root span
		spanCtx, span := tracer.Start(
			extractedCtx,
			info.FullMethod,
			trace.WithSpanKind(trace.SpanKindServer),
		)
		defer span.End()

		// Set standard RPC span attributes
		span.SetAttributes(
			semconv.RPCSystemGRPC,
			semconv.RPCService(info.FullMethod),
			semconv.RPCMethod(info.FullMethod),
		)

		// Extract and set peer IP if available
		if p, ok := peer.FromContext(ctx); ok {
			span.SetAttributes(
				attribute.String("net.peer.ip", p.Addr.String()),
			)
		}

		// Call the handler with the traced context
		resp, err := handler(spanCtx, req)

		// Record response status in span
		if err != nil {
			// Extract gRPC status code
			st, _ := status.FromError(err)
			span.SetAttributes(
				attribute.String("rpc.grpc.status_code", st.Code().String()),
			)
			span.RecordError(err)
			span.SetStatus(codes.Error, err.Error())
		} else {
			span.SetAttributes(
				attribute.String("rpc.grpc.status_code", "OK"),
			)
			span.SetStatus(codes.Ok, "")
		}

		return resp, err
	}
}
