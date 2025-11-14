module slate/services/metrics-service

go 1.21

require (
	github.com/google/uuid v1.5.0
	github.com/lib/pq v1.10.9
	github.com/prometheus/client_golang v1.18.0
	github.com/prometheus/common v0.45.0
	github.com/rs/zerolog v1.31.0
	go.opentelemetry.io/otel v1.21.0
	go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc v1.21.0
	go.opentelemetry.io/otel/sdk v1.21.0
	google.golang.org/grpc v1.60.1
	google.golang.org/protobuf v1.32.0
)
