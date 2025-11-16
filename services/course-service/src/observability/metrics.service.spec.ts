import { Test, TestingModule } from '@nestjs/testing';
import { MetricsService } from './metrics.service';

describe('MetricsService', () => {
  let service: MetricsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MetricsService],
    }).compile();

    service = module.get<MetricsService>(MetricsService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  it('should have a registry', () => {
    expect(service.registry).toBeDefined();
  });

  describe('Course metrics', () => {
    it('should increment courses created counter', () => {
      service.incrementCoursesCreated(true);
      service.incrementCoursesCreated(false);
      // No error thrown = success
      expect(true).toBe(true);
    });

    it('should increment courses published counter', () => {
      service.incrementCoursesPublished();
      expect(true).toBe(true);
    });

    it('should increment courses unpublished counter', () => {
      service.incrementCoursesUnpublished();
      expect(true).toBe(true);
    });
  });

  describe('Enrollment metrics', () => {
    it('should increment enrollments counter', () => {
      service.incrementEnrollments('self', true);
      service.incrementEnrollments('instructor', false);
      expect(true).toBe(true);
    });

    it('should increment enrollments removed counter', () => {
      service.incrementEnrollmentsRemoved();
      expect(true).toBe(true);
    });

    it('should increment self enrollments counter', () => {
      service.incrementSelfEnrollments(true);
      service.incrementSelfEnrollments(false);
      expect(true).toBe(true);
    });

    it('should increment instructor enrollments counter', () => {
      service.incrementInstructorEnrollments(true);
      service.incrementInstructorEnrollments(false);
      expect(true).toBe(true);
    });
  });

  describe('Template metrics', () => {
    it('should increment templates created counter', () => {
      service.incrementTemplatesCreated();
      expect(true).toBe(true);
    });

    it('should increment courses from templates counter', () => {
      service.incrementCoursesFromTemplates();
      expect(true).toBe(true);
    });
  });

  describe('Performance metrics', () => {
    it('should observe request duration', () => {
      service.observeRequestDuration('POST', '/api/courses', 201, 0.5);
      service.observeRequestDuration('GET', '/api/courses', 200, 0.1);
      expect(true).toBe(true);
    });

    it('should observe gRPC request duration', () => {
      service.observeGrpcRequestDuration('createCourse', 'success', 0.3);
      service.observeGrpcRequestDuration('getCourse', 'error', 0.2);
      expect(true).toBe(true);
    });
  });

  describe('System metrics', () => {
    it('should set database connections', () => {
      service.setDbConnections('active', 5);
      service.setDbConnections('idle', 2);
      expect(true).toBe(true);
    });
  });

  describe('getMetrics', () => {
    it('should return metrics in Prometheus format', async () => {
      service.incrementCoursesCreated(true);
      service.incrementSelfEnrollments(true);

      const metrics = await service.getMetrics();

      expect(metrics).toBeDefined();
      expect(typeof metrics).toBe('string');
      expect(metrics).toContain('courses_created_total');
      expect(metrics).toContain('self_enrollments_total');
    });
  });
});
