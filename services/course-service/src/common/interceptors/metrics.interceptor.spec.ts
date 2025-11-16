import { Test, TestingModule } from '@nestjs/testing';
import { ExecutionContext, CallHandler } from '@nestjs/common';
import { of, throwError } from 'rxjs';
import { MetricsInterceptor } from './metrics.interceptor';
import { MetricsService } from '../../observability/metrics.service';

describe('MetricsInterceptor', () => {
  let interceptor: MetricsInterceptor;
  let metricsService: jest.Mocked<MetricsService>;

  beforeEach(async () => {
    const mockMetricsService = {
      observeGrpcRequestDuration: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [MetricsInterceptor, { provide: MetricsService, useValue: mockMetricsService }],
    }).compile();

    interceptor = module.get<MetricsInterceptor>(MetricsInterceptor);
    metricsService = module.get(MetricsService);
  });

  const createMockExecutionContext = (methodName: string): ExecutionContext => {
    return {
      switchToRpc: jest.fn().mockReturnValue({
        getContext: jest.fn(),
        getData: jest.fn(),
      }),
      getHandler: jest.fn().mockReturnValue({ name: methodName }),
    } as any;
  };

  const createMockCallHandler = (result?: any, error?: Error): CallHandler => {
    return {
      handle: jest.fn().mockReturnValue(error ? throwError(() => error) : of(result)),
    } as any;
  };

  it('should be defined', () => {
    expect(interceptor).toBeDefined();
  });

  it('should track successful request duration', (done) => {
    const context = createMockExecutionContext('createCourse');
    const callHandler = createMockCallHandler({ success: true });

    interceptor.intercept(context, callHandler).subscribe(() => {
      expect(metricsService.observeGrpcRequestDuration).toHaveBeenCalledWith(
        'createCourse',
        'success',
        expect.any(Number),
      );
      done();
    });
  });

  it('should track failed request duration', (done) => {
    const context = createMockExecutionContext('getCourse');
    const error = new Error('Not found');
    const callHandler = createMockCallHandler(null, error);

    interceptor.intercept(context, callHandler).subscribe({
      error: () => {
        expect(metricsService.observeGrpcRequestDuration).toHaveBeenCalledWith(
          'getCourse',
          'error',
          expect.any(Number),
        );
        done();
      },
    });
  });

  it('should measure request duration accurately', (done) => {
    const context = createMockExecutionContext('listCourses');
    const callHandler = createMockCallHandler({ courses: [] });

    interceptor.intercept(context, callHandler).subscribe(() => {
      const recordedDuration = (metricsService.observeGrpcRequestDuration as jest.Mock).mock
        .calls[0][2];

      expect(recordedDuration).toBeGreaterThanOrEqual(0);
      expect(recordedDuration).toBeLessThan(1); // Should be very fast
      done();
    });
  });
});
