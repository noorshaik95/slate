import { Injectable, OnModuleInit } from '@nestjs/common';
import { Counter, Histogram, Registry, Gauge } from 'prom-client';

@Injectable()
export class MetricsService implements OnModuleInit {
  public readonly registry: Registry;

  // Course metrics
  private coursesCreated: Counter;
  private coursesPublished: Counter;
  private coursesUnpublished: Counter;

  // Enrollment metrics
  private enrollmentsCreated: Counter;
  private enrollmentsRemoved: Counter;
  private selfEnrollments: Counter;
  private instructorEnrollments: Counter;

  // Template metrics
  private templatesCreated: Counter;
  private coursesFromTemplates: Counter;

  // Performance metrics
  private requestDuration: Histogram;
  private grpcRequestDuration: Histogram;

  // System metrics
  private dbConnections: Gauge;

  constructor() {
    this.registry = new Registry();
    this.initializeMetrics();
  }

  onModuleInit() {
    // Set default labels
    this.registry.setDefaultLabels({
      service: 'course-service',
    });
  }

  private initializeMetrics() {
    // Course metrics
    this.coursesCreated = new Counter({
      name: 'courses_created_total',
      help: 'Total number of courses created',
      labelNames: ['success'],
      registers: [this.registry],
    });

    this.coursesPublished = new Counter({
      name: 'courses_published_total',
      help: 'Total number of courses published',
      registers: [this.registry],
    });

    this.coursesUnpublished = new Counter({
      name: 'courses_unpublished_total',
      help: 'Total number of courses unpublished',
      registers: [this.registry],
    });

    // Enrollment metrics
    this.enrollmentsCreated = new Counter({
      name: 'enrollments_created_total',
      help: 'Total number of enrollments created',
      labelNames: ['type', 'success'],
      registers: [this.registry],
    });

    this.enrollmentsRemoved = new Counter({
      name: 'enrollments_removed_total',
      help: 'Total number of enrollments removed',
      registers: [this.registry],
    });

    this.selfEnrollments = new Counter({
      name: 'self_enrollments_total',
      help: 'Total number of self-enrollments',
      labelNames: ['success'],
      registers: [this.registry],
    });

    this.instructorEnrollments = new Counter({
      name: 'instructor_enrollments_total',
      help: 'Total number of instructor-added enrollments',
      labelNames: ['success'],
      registers: [this.registry],
    });

    // Template metrics
    this.templatesCreated = new Counter({
      name: 'templates_created_total',
      help: 'Total number of course templates created',
      registers: [this.registry],
    });

    this.coursesFromTemplates = new Counter({
      name: 'courses_from_templates_total',
      help: 'Total number of courses created from templates',
      registers: [this.registry],
    });

    // Performance metrics
    this.requestDuration = new Histogram({
      name: 'http_request_duration_seconds',
      help: 'Duration of HTTP requests in seconds',
      labelNames: ['method', 'route', 'status_code'],
      buckets: [0.001, 0.01, 0.1, 0.5, 1, 2, 5],
      registers: [this.registry],
    });

    this.grpcRequestDuration = new Histogram({
      name: 'grpc_request_duration_seconds',
      help: 'Duration of gRPC requests in seconds',
      labelNames: ['method', 'status'],
      buckets: [0.001, 0.01, 0.1, 0.5, 1, 2, 5],
      registers: [this.registry],
    });

    // System metrics
    this.dbConnections = new Gauge({
      name: 'mongodb_connections',
      help: 'Number of MongoDB connections',
      labelNames: ['state'],
      registers: [this.registry],
    });
  }

  // Course metrics methods
  incrementCoursesCreated(success: boolean) {
    this.coursesCreated.inc({ success: success.toString() });
  }

  incrementCoursesPublished() {
    this.coursesPublished.inc();
  }

  incrementCoursesUnpublished() {
    this.coursesUnpublished.inc();
  }

  // Enrollment metrics methods
  incrementEnrollments(type: string, success: boolean) {
    this.enrollmentsCreated.inc({ type, success: success.toString() });
  }

  incrementEnrollmentsRemoved() {
    this.enrollmentsRemoved.inc();
  }

  incrementSelfEnrollments(success: boolean) {
    this.selfEnrollments.inc({ success: success.toString() });
  }

  incrementInstructorEnrollments(success: boolean) {
    this.instructorEnrollments.inc({ success: success.toString() });
  }

  // Template metrics methods
  incrementTemplatesCreated() {
    this.templatesCreated.inc();
  }

  incrementCoursesFromTemplates() {
    this.coursesFromTemplates.inc();
  }

  // Performance metrics methods
  observeRequestDuration(method: string, route: string, statusCode: number, duration: number) {
    this.requestDuration.observe({ method, route, status_code: statusCode }, duration);
  }

  observeGrpcRequestDuration(method: string, status: string, duration: number) {
    this.grpcRequestDuration.observe({ method, status }, duration);
  }

  // System metrics methods
  setDbConnections(state: string, count: number) {
    this.dbConnections.set({ state }, count);
  }

  getMetrics(): Promise<string> {
    return this.registry.metrics();
  }
}
