import {
  Injectable,
  Logger,
  NotFoundException,
  BadRequestException,
  ConflictException,
} from '@nestjs/common';
import { EnrollmentRepository } from './repositories/enrollment.repository';
import { CourseRepository } from '../course/repositories/course.repository';
import { SectionRepository } from '../course/repositories/section.repository';
import { MetricsService } from '../observability/metrics.service';
import { EnrollmentType, EnrollmentStatus } from './schemas/enrollment.schema';

@Injectable()
export class EnrollmentService {
  private readonly logger = new Logger(EnrollmentService.name);

  constructor(
    private readonly enrollmentRepository: EnrollmentRepository,
    private readonly courseRepository: CourseRepository,
    private readonly sectionRepository: SectionRepository,
    private readonly metricsService: MetricsService,
  ) {}

  async selfEnroll(courseId: string, studentId: string, sectionId?: string) {
    try {
      // Validate course exists and is published
      const course = await this.courseRepository.findById(courseId);
      if (!course) {
        throw new NotFoundException(`Course not found: ${courseId}`);
      }

      if (!course.isPublished) {
        throw new BadRequestException('Cannot enroll in unpublished course');
      }

      // Check if already enrolled
      const existing = await this.enrollmentRepository.findByCourseAndStudent(courseId, studentId);
      if (existing) {
        throw new ConflictException('Student is already enrolled in this course');
      }

      // Validate section if provided
      if (sectionId) {
        const section = await this.sectionRepository.findById(sectionId);
        if (!section) {
          throw new NotFoundException(`Section not found: ${sectionId}`);
        }

        // Check section capacity
        if (section.maxStudents > 0 && section.enrolledCount >= section.maxStudents) {
          throw new BadRequestException('Section is full');
        }
      }

      // Create enrollment
      const enrollment = await this.enrollmentRepository.create({
        courseId,
        studentId,
        enrollmentType: EnrollmentType.SELF,
        status: EnrollmentStatus.ACTIVE,
        enrolledBy: studentId,
        sectionId,
      });

      // Update section enrolled count if applicable
      if (sectionId) {
        await this.sectionRepository.incrementEnrolledCount(sectionId);
      }

      this.metricsService.incrementSelfEnrollments(true);
      this.metricsService.incrementEnrollments('self', true);
      this.logger.log(`Student ${studentId} self-enrolled in course ${courseId}`);

      return enrollment;
    } catch (error) {
      this.metricsService.incrementSelfEnrollments(false);
      this.metricsService.incrementEnrollments('self', false);
      this.logger.error(`Failed to self-enroll: ${error.message}`, error.stack);
      throw error;
    }
  }

  async instructorAddStudent(
    courseId: string,
    studentId: string,
    instructorId: string,
    sectionId?: string,
  ) {
    try {
      // Validate course exists
      const course = await this.courseRepository.findById(courseId);
      if (!course) {
        throw new NotFoundException(`Course not found: ${courseId}`);
      }

      // Verify instructor has permission
      const isInstructor =
        course.instructorId === instructorId || course.coInstructorIds.includes(instructorId);

      if (!isInstructor) {
        throw new BadRequestException('Only course instructors can add students');
      }

      // Check if already enrolled
      const existing = await this.enrollmentRepository.findByCourseAndStudent(courseId, studentId);
      if (existing) {
        throw new ConflictException('Student is already enrolled in this course');
      }

      // Validate section if provided
      if (sectionId) {
        const section = await this.sectionRepository.findById(sectionId);
        if (!section) {
          throw new NotFoundException(`Section not found: ${sectionId}`);
        }
      }

      // Create enrollment
      const enrollment = await this.enrollmentRepository.create({
        courseId,
        studentId,
        enrollmentType: EnrollmentType.INSTRUCTOR,
        status: EnrollmentStatus.ACTIVE,
        enrolledBy: instructorId,
        sectionId,
      });

      // Update section enrolled count if applicable
      if (sectionId) {
        await this.sectionRepository.incrementEnrolledCount(sectionId);
      }

      this.metricsService.incrementInstructorEnrollments(true);
      this.metricsService.incrementEnrollments('instructor', true);
      this.logger.log(
        `Instructor ${instructorId} added student ${studentId} to course ${courseId}`,
      );

      return enrollment;
    } catch (error) {
      this.metricsService.incrementInstructorEnrollments(false);
      this.metricsService.incrementEnrollments('instructor', false);
      this.logger.error(`Failed to add student: ${error.message}`, error.stack);
      throw error;
    }
  }

  async removeEnrollment(enrollmentId: string) {
    const enrollment = await this.enrollmentRepository.findById(enrollmentId);
    if (!enrollment) {
      throw new NotFoundException(`Enrollment not found: ${enrollmentId}`);
    }

    // Update section enrolled count if applicable
    if (enrollment.sectionId) {
      await this.sectionRepository.decrementEnrolledCount(enrollment.sectionId.toString());
    }

    await this.enrollmentRepository.delete(enrollmentId);
    this.metricsService.incrementEnrollmentsRemoved();
    this.logger.log(`Enrollment removed: ${enrollmentId}`);

    return true;
  }

  async getCourseRoster(courseId: string, sectionId?: string, status?: EnrollmentStatus) {
    // Validate course exists
    const course = await this.courseRepository.findById(courseId);
    if (!course) {
      throw new NotFoundException(`Course not found: ${courseId}`);
    }

    const enrollments = await this.enrollmentRepository.findByCourse(courseId, {
      sectionId,
      status,
    });

    return {
      enrollments,
      totalCount: enrollments.length,
    };
  }

  async getStudentEnrollments(studentId: string, term?: string, status?: EnrollmentStatus) {
    const enrollments = await this.enrollmentRepository.findByStudent(studentId, {
      term,
      status,
    });

    // Populate course information
    const enrollmentsWithCourses = await Promise.all(
      enrollments.map(async (enrollment) => {
        const course = await this.courseRepository.findById(enrollment.courseId.toString());
        return {
          enrollment,
          course,
        };
      }),
    );

    return enrollmentsWithCourses;
  }
}
