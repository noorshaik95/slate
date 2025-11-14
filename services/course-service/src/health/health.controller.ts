import { Controller, Get } from '@nestjs/common';
import { InjectConnection } from '@nestjs/mongoose';
import { Connection } from 'mongoose';

@Controller('health')
export class HealthController {
  constructor(@InjectConnection() private readonly connection: Connection) {}

  @Get()
  async check() {
    const dbStatus = this.connection.readyState === 1 ? 'healthy' : 'unhealthy';

    return {
      status: dbStatus === 'healthy' ? 'healthy' : 'unhealthy',
      service: 'course-service',
      timestamp: new Date().toISOString(),
      checks: {
        database: dbStatus,
      },
    };
  }

  @Get('liveness')
  liveness() {
    return {
      status: 'alive',
      timestamp: new Date().toISOString(),
    };
  }

  @Get('readiness')
  async readiness() {
    const dbReady = this.connection.readyState === 1;

    return {
      status: dbReady ? 'ready' : 'not_ready',
      timestamp: new Date().toISOString(),
      checks: {
        database: dbReady,
      },
    };
  }
}
