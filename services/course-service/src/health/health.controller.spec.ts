import { Test, TestingModule } from '@nestjs/testing';
import { getConnectionToken } from '@nestjs/mongoose';
import { HealthController } from './health.controller';

describe('HealthController', () => {
  let controller: HealthController;
  let mockConnection: any;

  beforeEach(async () => {
    mockConnection = {
      readyState: 1, // Connected
    };

    const module: TestingModule = await Test.createTestingModule({
      controllers: [HealthController],
      providers: [
        {
          provide: getConnectionToken(),
          useValue: mockConnection,
        },
      ],
    }).compile();

    controller = module.get<HealthController>(HealthController);
  });

  describe('check', () => {
    it('should return healthy status when database is connected', async () => {
      mockConnection.readyState = 1;

      const result = await controller.check();

      expect(result.status).toBe('healthy');
      expect(result.service).toBe('course-service');
      expect(result.checks.database).toBe('healthy');
      expect(result.timestamp).toBeDefined();
    });

    it('should return unhealthy status when database is disconnected', async () => {
      mockConnection.readyState = 0;

      const result = await controller.check();

      expect(result.status).toBe('unhealthy');
      expect(result.checks.database).toBe('unhealthy');
    });

    it('should return unhealthy status when database is connecting', async () => {
      mockConnection.readyState = 2;

      const result = await controller.check();

      expect(result.status).toBe('unhealthy');
      expect(result.checks.database).toBe('unhealthy');
    });
  });

  describe('liveness', () => {
    it('should return alive status', () => {
      const result = controller.liveness();

      expect(result.status).toBe('alive');
      expect(result.timestamp).toBeDefined();
    });
  });

  describe('readiness', () => {
    it('should return ready status when database is connected', async () => {
      mockConnection.readyState = 1;

      const result = await controller.readiness();

      expect(result.status).toBe('ready');
      expect(result.checks.database).toBe(true);
    });

    it('should return not ready status when database is disconnected', async () => {
      mockConnection.readyState = 0;

      const result = await controller.readiness();

      expect(result.status).toBe('not_ready');
      expect(result.checks.database).toBe(false);
    });
  });
});
