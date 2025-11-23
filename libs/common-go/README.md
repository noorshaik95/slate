# Common Go Library

Shared utilities and helper functions for Go microservices in the Slate platform.

## Packages

### tracing

OpenTelemetry tracing utilities for function-level instrumentation.

## Installation

Add to your service's `go.mod`:

```go
require (
    slate/libs/common-go v0.0.0
)

replace slate/libs/common-go => ../../libs/common-go
```

Then run:

```bash
go mod tidy
```

## Usage

### Tracing Package

The tracing package provides helper functions to create and manage OpenTelemetry spans with minimal boilerplate.

#### Import

```go
import (
    "slate/libs/common-go/tracing"
    "go.opentelemetry.io/otel/attribute"
)
```

#### Basic Span Creation

```go
func MyHandler(ctx context.Context, req *Request) error {
    ctx, span := tracing.StartSpan(ctx, "my_handler",
        attribute.String("request_id", req.ID))
    defer span.End()

    // Your handler logic
    return processRequest(ctx, req)
}
```

#### Automatic Error Recording

Use the defer pattern with named return values:

```go
func MyService(ctx context.Context, id string) (result *Data, err error) {
    ctx, span := tracing.StartSpan(ctx, "my_service",
        attribute.String("id", id))
    defer tracing.EndSpanWithError(span, &err)

    // Your service logic
    result, err = fetchData(ctx, id)
    if err != nil {
        return nil, err
    }

    return result, nil
}
```

#### Setting Service Name

Set the tracer name in your service initialization:

```go
func main() {
    ctx := context.Background()
    ctx = tracing.WithTracerName(ctx, "user-auth-service")

    // Pass ctx to your handlers
    server := NewServer(ctx)
    server.Start()
}
```

#### Nested Spans

Create child spans for detailed tracing:

```go
func ProcessOrder(ctx context.Context, order *Order) (err error) {
    ctx, span := tracing.StartSpan(ctx, "process_order",
        attribute.String("order_id", order.ID))
    defer tracing.EndSpanWithError(span, &err)

    // Validate order
    ctx, validateSpan := tracing.StartSpan(ctx, "validate_order")
    if err := validateOrder(ctx, order); err != nil {
        tracing.RecordError(validateSpan, err, "validation failed")
        validateSpan.End()
        return err
    }
    validateSpan.End()

    // Save order
    ctx, saveSpan := tracing.StartSpan(ctx, "save_order")
    defer tracing.EndSpanWithError(saveSpan, &err)
    
    return saveToDatabase(ctx, order)
}
```

#### Recording Errors Without Ending Span

```go
func ProcessBatch(ctx context.Context, items []Item) error {
    ctx, span := tracing.StartSpan(ctx, "process_batch",
        attribute.Int("count", len(items)))
    defer span.End()

    for _, item := range items {
        if err := processItem(ctx, item); err != nil {
            // Record error but continue processing
            tracing.RecordError(span, err, "item processing failed")
            continue
        }
    }

    tracing.SetSpanStatus(span, codes.Ok, "batch completed")
    return nil
}
```

### Naming Conventions

Follow these conventions for consistent tracing:

| Layer | Pattern | Example |
|-------|---------|---------|
| Handler | `{operation}_handler` | `login_handler`, `create_user_handler` |
| Service | `{service}.{operation}` | `user_service.login`, `order_service.create` |
| Repository | `db.{operation}` | `db.get_user`, `db.create_order` |
| Utility | `{operation}` | `validate_email`, `hash_password` |

### Attribute Guidelines

**Include:**
- Resource identifiers: `user_id`, `order_id`, `request_id`
- Operation metadata: `method`, `table`, `operation_type`
- Non-sensitive data: `email` (but not password)
- Result indicators: `found`, `count`, `status`

**Exclude:**
- Passwords or password hashes
- Authentication tokens
- API keys or secrets
- Large request/response bodies
- Sensitive PII

### Example: Complete Handler Implementation

```go
package grpc

import (
    "context"
    "slate/libs/common-go/tracing"
    "go.opentelemetry.io/otel/attribute"
    "go.opentelemetry.io/otel/codes"
)

func (s *UserServiceServer) Login(ctx context.Context, req *pb.LoginRequest) (*pb.LoginResponse, error) {
    // Set tracer name
    ctx = tracing.WithTracerName(ctx, "user-auth-service")
    
    // Create handler span
    ctx, span := tracing.StartSpan(ctx, "login_handler",
        attribute.String("email", req.Email),
        attribute.String("method", "Login"))
    defer span.End()
    
    // Call service layer
    user, tokens, err := s.userService.Login(ctx, req.Email, req.Password)
    if err != nil {
        span.RecordError(err)
        span.SetStatus(codes.Error, "login failed")
        return nil, status.Errorf(codes.Unauthenticated, "login failed: %v", err)
    }
    
    span.SetStatus(codes.Ok, "")
    return &pb.LoginResponse{
        AccessToken:  tokens.AccessToken,
        RefreshToken: tokens.RefreshToken,
        User:         userToProto(user),
    }, nil
}
```

## Development

### Running Tests

```bash
cd libs/common-go
go test ./...
```

### Building

```bash
cd libs/common-go
go build ./...
```

## Contributing

When adding new utilities:

1. Add comprehensive documentation
2. Include usage examples
3. Write unit tests
4. Update this README

## License

Internal use only - Slate Platform
