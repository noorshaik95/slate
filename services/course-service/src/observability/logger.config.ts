import { Params } from 'nestjs-pino';

export const loggerConfig: Params = {
  pinoHttp: {
    level: process.env.LOG_LEVEL || 'info',
    transport:
      process.env.NODE_ENV !== 'production'
        ? {
            target: 'pino-pretty',
            options: {
              colorize: true,
              levelFirst: true,
              translateTime: 'UTC:yyyy-mm-dd HH:MM:ss.l o',
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
