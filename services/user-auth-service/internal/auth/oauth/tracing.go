package oauth

import "net/http"

// httpHeaderCarrier adapts http.Header to be used as a TextMapCarrier for trace context propagation.
// This allows OpenTelemetry to inject trace context (traceparent, tracestate) into HTTP headers
// for distributed tracing across service boundaries.
type httpHeaderCarrier struct {
	header http.Header
}

// Get retrieves a value from the HTTP header by key
func (hc *httpHeaderCarrier) Get(key string) string {
	return hc.header.Get(key)
}

// Set sets a value in the HTTP header
func (hc *httpHeaderCarrier) Set(key, value string) {
	hc.header.Set(key, value)
}

// Keys returns all keys in the HTTP header
func (hc *httpHeaderCarrier) Keys() []string {
	keys := make([]string, 0, len(hc.header))
	for k := range hc.header {
		keys = append(keys, k)
	}
	return keys
}
