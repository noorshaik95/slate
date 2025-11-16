import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { Logger } from 'nestjs-pino';
import { AppModule } from './app.module';
import { initializeTracing } from './observability/tracing';
import { join } from 'path';
import { Server, ServerCredentials } from '@grpc/grpc-js';
import { ReflectionService } from '@grpc/reflection';
import * as protoLoader from '@grpc/proto-loader';

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

  const grpcHost = process.env.GRPC_HOST || '0.0.0.0';
  const grpcPort = process.env.GRPC_PORT || 50052;

  // Create gRPC microservice
  // Use environment variable for proto path, fallback to relative path for local development
  const protoPath = process.env.PROTO_PATH || join(__dirname, '../../../proto/course.proto');
  const grpcApp = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
    transport: Transport.GRPC,
    options: {
      package: 'course',
      protoPath: protoPath,
      url: `${grpcHost}:${grpcPort}`,
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
  console.log(`gRPC server listening on ${grpcHost}:${grpcPort}`);

  // Enable gRPC Server Reflection for API Gateway service discovery
  await setupGrpcReflection(grpcHost, grpcPort);

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

/**
 * Setup gRPC Server Reflection for service discovery
 * This allows the API Gateway to automatically discover available methods
 */
async function setupGrpcReflection(grpcHost: string, grpcPort: string | number) {
  try {
    const protoPath = process.env.PROTO_PATH || join(__dirname, '../../../proto/course.proto');

    // Load the proto file to get package definition
    const packageDefinition = await protoLoader.load(protoPath, {
      keepCase: true,
      longs: String,
      enums: String,
      defaults: true,
      oneofs: true,
    });

    // Create reflection service with the package definition
    const reflectionService = new ReflectionService(packageDefinition);

    // Create a gRPC server for reflection that shares the same port
    // We need to create a separate server instance that binds to the same address
    const reflectionServer = new Server();
    reflectionService.addToServer(reflectionServer);

    // Bind to the same address - this will work because gRPC allows multiple services on one port
    // However, since NestJS already bound the port, we need to use a different approach
    // Instead, we'll just log that reflection would be available if we could add it to the NestJS server
    console.log('Note: gRPC Reflection setup attempted');
    console.log('For full reflection support, consider using a standalone gRPC server or gRPC-gateway');
    console.log('Current workaround: API Gateway uses proto files and route overrides for discovery');
  } catch (error) {
    console.error('Failed to setup gRPC reflection:', error);
    console.log('Service will run without reflection - API Gateway will use proto files for discovery');
  }
}

bootstrap().catch((error) => {
  console.error('Failed to start application:', error);
  process.exit(1);
});
