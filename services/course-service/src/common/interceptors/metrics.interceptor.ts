import {
  Injectable,
  NestInterceptor,
  ExecutionContext,
  CallHandler,
} from '@nestjs/common';
import { Observable } from 'rxjs';
import { tap } from 'rxjs/operators';
import { MetricsService } from '../../observability/metrics.service';

@Injectable()
export class MetricsInterceptor implements NestInterceptor {
  constructor(private readonly metricsService: MetricsService) {}

  intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
    const start = Date.now();
    const rpcContext = context.switchToRpc();
    const methodName = context.getHandler().name;

    return next.handle().pipe(
      tap({
        next: () => {
          const duration = (Date.now() - start) / 1000;
          this.metricsService.observeGrpcRequestDuration(methodName, 'success', duration);
        },
        error: () => {
          const duration = (Date.now() - start) / 1000;
          this.metricsService.observeGrpcRequestDuration(methodName, 'error', duration);
        },
      }),
    );
  }
}
