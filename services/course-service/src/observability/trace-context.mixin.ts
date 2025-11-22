import { trace, context, SpanContext } from '@opentelemetry/api';

/**
 * Pino mixin function that extracts trace context from the active OpenTelemetry span
 * and adds it to every log entry.
 *
 * This enables correlation between distributed traces (in Tempo) and application logs (in Loki)
 * by automatically injecting trace_id, span_id, and trace_flags into all log entries.
 *
 * @returns Object containing trace_id, span_id, and trace_flags if an active span exists,
 *          or an empty object if no active span is present (graceful degradation)
 */
export function traceContextMixin() {
  try {
    const span = trace.getSpan(context.active());

    if (!span) {
      return {};
    }

    const spanContext: SpanContext = span.spanContext();

    // Only include trace context if the span is valid and has a trace ID
    if (!spanContext || !spanContext.traceId) {
      return {};
    }

    return {
      trace_id: spanContext.traceId,
      span_id: spanContext.spanId,
      trace_flags: spanContext.traceFlags,
    };
  } catch (error) {
    // Fail silently - don't break logging if trace extraction fails
    console.error('Error extracting trace context for logs:', error);
    return {};
  }
}
