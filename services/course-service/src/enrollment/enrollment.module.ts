import { Module } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';
import { EnrollmentService } from './enrollment.service';
import { EnrollmentRepository } from './repositories/enrollment.repository';
import { Enrollment, EnrollmentSchema } from './schemas/enrollment.schema';
import { MetricsService } from '../observability/metrics.service';
import { CourseRepository } from '../course/repositories/course.repository';
import { SectionRepository } from '../course/repositories/section.repository';
import { Course, CourseSchema } from '../course/schemas/course.schema';
import { Section, SectionSchema } from '../course/schemas/section.schema';

@Module({
  imports: [
    MongooseModule.forFeature([
      { name: Enrollment.name, schema: EnrollmentSchema },
      { name: Course.name, schema: CourseSchema },
      { name: Section.name, schema: SectionSchema },
    ]),
  ],
  providers: [
    EnrollmentService,
    EnrollmentRepository,
    CourseRepository,
    SectionRepository,
    MetricsService,
  ],
  exports: [EnrollmentService, EnrollmentRepository],
})
export class EnrollmentModule {}
