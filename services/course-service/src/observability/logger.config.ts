import { Params } from 'nestjs-pino';
import { traceContextMixin } from './trace-context.mixin';

export const loggerConfig: Params = {
  pinoHttp: {
    level: process.env.LOG_LEVEL || 'info',
    
    // Add trace context mixin to inject trace_id, span_id, trace_flags into all logs
    mixin: traceContextMixin,
    
    // Exclude health check endpoints from automatic request logging to reduce log noise
    autoLogging: {
      ignore: (req) => req.url === '/health',
    },
    
    transport:
      process.env.NODE_ENV !== 'production'
        ? {
            target: 'pino-pretty',
            options: {
              colorize: true,
              levelFirst: true,
              translateTime: 'UTC:yyyy-mm-dd HH:MM:ss.l o',
              // Show trace fields in pretty output, hide noisy fields
              ignore: 'pid,hostname',
            },
          }
        : undefined,
    serializers: {
      req: (req) => ({
        id: req.id,
        method: req.method,
        url: req.url,
      }),
      res: (res) => ({
        statusCode: res.statusCode,
      }),
    },
    redact: {
      paths: [
        'req.headers.authorization',
        'req.headers.cookie',
        '*.password',
        '*.token',
        '*.accessToken',
        '*.refreshToken',
      ],
      remove: true,
    },
    formatters: {
      level: (label: string) => {
        return { level: label };
      },
    },
    base: {
      service: 'course-service',
      version: process.env.SERVICE_VERSION || '1.0.0',
    },
  },
};
