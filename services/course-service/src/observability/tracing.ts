import { NodeSDK } from '@opentelemetry/sdk-node';
import { getNodeAutoInstrumentations } from '@opentelemetry/auto-instrumentations-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';
import { Resource } from '@opentelemetry/resources';
import { SemanticResourceAttributes } from '@opentelemetry/semantic-conventions';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-base';

export function initializeTracing() {
  const serviceName = 'course-service';
  const serviceVersion = process.env.SERVICE_VERSION || '1.0.0';
  const otlpEndpoint = process.env.OTEL_EXPORTER_OTLP_ENDPOINT || 'http://tempo:4317';

  const resource = new Resource({
    [SemanticResourceAttributes.SERVICE_NAME]: serviceName,
    [SemanticResourceAttributes.SERVICE_VERSION]: serviceVersion,
    [SemanticResourceAttributes.DEPLOYMENT_ENVIRONMENT]: process.env.NODE_ENV || 'development',
  });

  const traceExporter = new OTLPTraceExporter({
    url: otlpEndpoint,
  });

  const sdk = new NodeSDK({
    resource,
    traceExporter,
    spanProcessor: new BatchSpanProcessor(traceExporter),
    instrumentations: [
      getNodeAutoInstrumentations({
        '@opentelemetry/instrumentation-fs': {
          enabled: false, // Disable fs instrumentation to reduce noise
        },
        '@opentelemetry/instrumentation-mongodb': {
          enabled: true,
        },
        '@opentelemetry/instrumentation-http': {
          enabled: true,
        },
        '@opentelemetry/instrumentation-grpc': {
          enabled: true,
        },
      }),
    ],
  });

  sdk.start();

  // Graceful shutdown
  process.on('SIGTERM', () => {
    sdk
      .shutdown()
      .then(() => console.log('Tracing terminated'))
      .catch((error) => console.error('Error terminating tracing', error))
      .finally(() => process.exit(0));
  });

  return sdk;
}
