import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { Logger } from 'nestjs-pino';
import { AppModule } from './app.module';
import { initializeTracing } from './observability/tracing';
import { join } from 'path';

// Initialize OpenTelemetry tracing before anything else
initializeTracing();

async function bootstrap() {
  // Create HTTP app for health checks and metrics
  const app = await NestFactory.create(AppModule, { bufferLogs: true });

  // Use Pino logger
  app.useLogger(app.get(Logger));

  // Enable graceful shutdown
  app.enableShutdownHooks();

  const port = process.env.PORT || 3000;
  await app.listen(port);
  console.log(`HTTP server listening on port ${port} (health and metrics)`);

  // Create gRPC microservice
  const grpcApp = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
    transport: Transport.GRPC,
    options: {
      package: 'course',
      protoPath: join(__dirname, '../../../proto/course.proto'),
      url: `${process.env.GRPC_HOST || '0.0.0.0'}:${process.env.GRPC_PORT || 50052}`,
      loader: {
        keepCase: true,
        longs: String,
        enums: String,
        defaults: true,
        oneofs: true,
      },
    },
  });

  grpcApp.useLogger(grpcApp.get(Logger));

  await grpcApp.listen();
  console.log(
    `gRPC server listening on ${process.env.GRPC_HOST || '0.0.0.0'}:${process.env.GRPC_PORT || 50052}`,
  );

  // Graceful shutdown handlers
  const shutdown = async (signal: string) => {
    console.log(`Received ${signal}, starting graceful shutdown...`);
    try {
      await app.close();
      await grpcApp.close();
      console.log('Graceful shutdown completed');
      process.exit(0);
    } catch (error) {
      console.error('Error during shutdown:', error);
      process.exit(1);
    }
  };

  process.on('SIGTERM', () => shutdown('SIGTERM'));
  process.on('SIGINT', () => shutdown('SIGINT'));
}

bootstrap().catch((error) => {
  console.error('Failed to start application:', error);
  process.exit(1);
});
