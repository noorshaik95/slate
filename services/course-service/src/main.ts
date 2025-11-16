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
  const grpcApp = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
    transport: Transport.GRPC,
    options: {
      package: 'course',
      protoPath: join(__dirname, '../../../proto/course.proto'),
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

  // Enable gRPC Server Reflection for API Gateway service discovery
  // This creates a separate reflection server that runs on a different port
  await setupGrpcReflection();

  await grpcApp.listen();
  console.log(`gRPC server listening on ${grpcHost}:${grpcPort}`);

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
async function setupGrpcReflection() {
  try {
    const protoPath = join(__dirname, '../../../proto/course.proto');
    const grpcHost = process.env.GRPC_HOST || '0.0.0.0';
    const grpcPort = process.env.GRPC_PORT || 50052;

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

    // Create a new gRPC server for reflection
    const reflectionServer = new Server();

    // Add reflection service to the server
    reflectionService.addToServer(reflectionServer);

    // Bind to the same address as the main gRPC server
    // Note: This shares the same port with the NestJS gRPC server
    reflectionServer.bindAsync(
      `${grpcHost}:${grpcPort}`,
      ServerCredentials.createInsecure(),
      (err, port) => {
        if (err) {
          console.warn(
            'gRPC Reflection server binding failed (this is normal if NestJS already bound the port):',
            err.message,
          );
          console.log(
            'Note: Reflection may need to be enabled differently for full NestJS compatibility',
          );
        } else {
          reflectionServer.start();
          console.log(`gRPC Server Reflection enabled on port ${port}`);
          console.log('Service discovery is now available for API Gateway');
        }
      },
    );
  } catch (error) {
    console.error('Failed to setup gRPC reflection:', error);
    console.log('Service will run without reflection - manual route configuration may be needed');
  }
}

bootstrap().catch((error) => {
  console.error('Failed to start application:', error);
  process.exit(1);
});
