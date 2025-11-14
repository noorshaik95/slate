import { Test, TestingModule } from '@nestjs/testing';
import { NotFoundException, BadRequestException, ConflictException } from '@nestjs/common';
import { EnrollmentService } from './enrollment.service';
import { EnrollmentRepository } from './repositories/enrollment.repository';
import { CourseRepository } from '../course/repositories/course.repository';
import { SectionRepository } from '../course/repositories/section.repository';
import { MetricsService } from '../observability/metrics.service';
import { EnrollmentType, EnrollmentStatus } from './schemas/enrollment.schema';

describe('EnrollmentService', () => {
  let service: EnrollmentService;
  let enrollmentRepository: jest.Mocked<EnrollmentRepository>;
  let courseRepository: jest.Mocked<CourseRepository>;
  let sectionRepository: jest.Mocked<SectionRepository>;
  let metricsService: jest.Mocked<MetricsService>;

  const mockCourse = {
    _id: 'course-1',
    title: 'Test Course',
    description: 'Test Description',
    term: 'Fall 2024',
    instructorId: 'instructor-1',
    coInstructorIds: [],
    isPublished: true,
    prerequisiteCourseIds: [],
  };

  const mockSection = {
    _id: 'section-1',
    courseId: 'course-1',
    sectionNumber: '001',
    instructorId: 'instructor-1',
    maxStudents: 30,
    enrolledCount: 10,
  };

  const mockEnrollment = {
    _id: 'enrollment-1',
    courseId: 'course-1',
    studentId: 'student-1',
    enrollmentType: EnrollmentType.SELF,
    status: EnrollmentStatus.ACTIVE,
    enrolledBy: 'student-1',
    enrolledAt: new Date(),
  };

  beforeEach(async () => {
    const mockEnrollmentRepo = {
      create: jest.fn(),
      findById: jest.fn(),
      findByCourseAndStudent: jest.fn(),
      findByCourse: jest.fn(),
      findByStudent: jest.fn(),
      delete: jest.fn(),
      updateStatus: jest.fn(),
      countByCourse: jest.fn(),
      countBySection: jest.fn(),
    };

    const mockCourseRepo = {
      findById: jest.fn(),
    };

    const mockSectionRepo = {
      findById: jest.fn(),
      incrementEnrolledCount: jest.fn(),
      decrementEnrolledCount: jest.fn(),
    };

    const mockMetricsService = {
      incrementEnrollments: jest.fn(),
      incrementEnrollmentsRemoved: jest.fn(),
      incrementSelfEnrollments: jest.fn(),
      incrementInstructorEnrollments: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        EnrollmentService,
        { provide: EnrollmentRepository, useValue: mockEnrollmentRepo },
        { provide: CourseRepository, useValue: mockCourseRepo },
        { provide: SectionRepository, useValue: mockSectionRepo },
        { provide: MetricsService, useValue: mockMetricsService },
      ],
    }).compile();

    service = module.get<EnrollmentService>(EnrollmentService);
    enrollmentRepository = module.get(EnrollmentRepository);
    courseRepository = module.get(CourseRepository);
    sectionRepository = module.get(SectionRepository);
    metricsService = module.get(MetricsService);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('selfEnroll', () => {
    it('should allow self-enrollment in published course', async () => {
      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourseAndStudent.mockResolvedValue(null);
      enrollmentRepository.create.mockResolvedValue(mockEnrollment as any);

      const result = await service.selfEnroll('course-1', 'student-1');

      expect(courseRepository.findById).toHaveBeenCalledWith('course-1');
      expect(enrollmentRepository.findByCourseAndStudent).toHaveBeenCalledWith(
        'course-1',
        'student-1',
      );
      expect(enrollmentRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          courseId: 'course-1',
          studentId: 'student-1',
          enrollmentType: EnrollmentType.SELF,
          status: EnrollmentStatus.ACTIVE,
          enrolledBy: 'student-1',
        }),
      );
      expect(metricsService.incrementSelfEnrollments).toHaveBeenCalledWith(true);
      expect(metricsService.incrementEnrollments).toHaveBeenCalledWith('self', true);
      expect(result).toEqual(mockEnrollment);
    });

    it('should throw NotFoundException when course does not exist', async () => {
      courseRepository.findById.mockResolvedValue(null);

      await expect(service.selfEnroll('non-existent-course', 'student-1')).rejects.toThrow(
        NotFoundException,
      );
      await expect(service.selfEnroll('non-existent-course', 'student-1')).rejects.toThrow(
        'Course not found: non-existent-course',
      );

      expect(metricsService.incrementSelfEnrollments).toHaveBeenCalledWith(false);
    });

    it('should throw BadRequestException when course is not published', async () => {
      const unpublishedCourse = { ...mockCourse, isPublished: false };
      courseRepository.findById.mockResolvedValue(unpublishedCourse as any);

      await expect(service.selfEnroll('course-1', 'student-1')).rejects.toThrow(
        BadRequestException,
      );
      await expect(service.selfEnroll('course-1', 'student-1')).rejects.toThrow(
        'Cannot enroll in unpublished course',
      );

      expect(metricsService.incrementSelfEnrollments).toHaveBeenCalledWith(false);
    });

    it('should throw ConflictException when student already enrolled', async () => {
      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourseAndStudent.mockResolvedValue(mockEnrollment as any);

      await expect(service.selfEnroll('course-1', 'student-1')).rejects.toThrow(
        ConflictException,
      );
      await expect(service.selfEnroll('course-1', 'student-1')).rejects.toThrow(
        'Student is already enrolled in this course',
      );

      expect(metricsService.incrementSelfEnrollments).toHaveBeenCalledWith(false);
    });

    it('should enroll in section and update section count', async () => {
      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourseAndStudent.mockResolvedValue(null);
      sectionRepository.findById.mockResolvedValue(mockSection as any);
      enrollmentRepository.create.mockResolvedValue({ ...mockEnrollment, sectionId: 'section-1' } as any);
      sectionRepository.incrementEnrolledCount.mockResolvedValue(mockSection as any);

      const result = await service.selfEnroll('course-1', 'student-1', 'section-1');

      expect(sectionRepository.findById).toHaveBeenCalledWith('section-1');
      expect(sectionRepository.incrementEnrolledCount).toHaveBeenCalledWith('section-1');
      expect(result.sectionId).toBe('section-1');
    });

    it('should throw BadRequestException when section is full', async () => {
      const fullSection = { ...mockSection, enrolledCount: 30, maxStudents: 30 };
      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourseAndStudent.mockResolvedValue(null);
      sectionRepository.findById.mockResolvedValue(fullSection as any);

      await expect(service.selfEnroll('course-1', 'student-1', 'section-1')).rejects.toThrow(
        BadRequestException,
      );
      await expect(service.selfEnroll('course-1', 'student-1', 'section-1')).rejects.toThrow(
        'Section is full',
      );
    });

    it('should throw NotFoundException when section does not exist', async () => {
      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourseAndStudent.mockResolvedValue(null);
      sectionRepository.findById.mockResolvedValue(null);

      await expect(service.selfEnroll('course-1', 'student-1', 'non-existent-section')).rejects.toThrow(
        NotFoundException,
      );
    });
  });

  describe('instructorAddStudent', () => {
    it('should allow instructor to add student', async () => {
      const instructorEnrollment = {
        ...mockEnrollment,
        enrollmentType: EnrollmentType.INSTRUCTOR,
        enrolledBy: 'instructor-1',
      };

      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourseAndStudent.mockResolvedValue(null);
      enrollmentRepository.create.mockResolvedValue(instructorEnrollment as any);

      const result = await service.instructorAddStudent(
        'course-1',
        'student-1',
        'instructor-1',
      );

      expect(enrollmentRepository.create).toHaveBeenCalledWith(
        expect.objectContaining({
          courseId: 'course-1',
          studentId: 'student-1',
          enrollmentType: EnrollmentType.INSTRUCTOR,
          enrolledBy: 'instructor-1',
        }),
      );
      expect(metricsService.incrementInstructorEnrollments).toHaveBeenCalledWith(true);
      expect(result).toEqual(instructorEnrollment);
    });

    it('should allow co-instructor to add student', async () => {
      const courseWithCoInstructor = {
        ...mockCourse,
        coInstructorIds: ['co-instructor-1'],
      };

      courseRepository.findById.mockResolvedValue(courseWithCoInstructor as any);
      enrollmentRepository.findByCourseAndStudent.mockResolvedValue(null);
      enrollmentRepository.create.mockResolvedValue(mockEnrollment as any);

      await service.instructorAddStudent('course-1', 'student-1', 'co-instructor-1');

      expect(enrollmentRepository.create).toHaveBeenCalled();
      expect(metricsService.incrementInstructorEnrollments).toHaveBeenCalledWith(true);
    });

    it('should throw BadRequestException when user is not instructor', async () => {
      courseRepository.findById.mockResolvedValue(mockCourse as any);

      await expect(
        service.instructorAddStudent('course-1', 'student-1', 'random-user'),
      ).rejects.toThrow(BadRequestException);
      await expect(
        service.instructorAddStudent('course-1', 'student-1', 'random-user'),
      ).rejects.toThrow('Only course instructors can add students');

      expect(metricsService.incrementInstructorEnrollments).toHaveBeenCalledWith(false);
    });

    it('should throw ConflictException when student already enrolled', async () => {
      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourseAndStudent.mockResolvedValue(mockEnrollment as any);

      await expect(
        service.instructorAddStudent('course-1', 'student-1', 'instructor-1'),
      ).rejects.toThrow(ConflictException);
    });

    it('should add student to section when section provided', async () => {
      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourseAndStudent.mockResolvedValue(null);
      sectionRepository.findById.mockResolvedValue(mockSection as any);
      enrollmentRepository.create.mockResolvedValue({ ...mockEnrollment, sectionId: 'section-1' } as any);

      await service.instructorAddStudent('course-1', 'student-1', 'instructor-1', 'section-1');

      expect(sectionRepository.incrementEnrolledCount).toHaveBeenCalledWith('section-1');
    });
  });

  describe('removeEnrollment', () => {
    it('should remove enrollment successfully', async () => {
      enrollmentRepository.findById.mockResolvedValue(mockEnrollment as any);
      enrollmentRepository.delete.mockResolvedValue(true);

      const result = await service.removeEnrollment('enrollment-1');

      expect(enrollmentRepository.findById).toHaveBeenCalledWith('enrollment-1');
      expect(enrollmentRepository.delete).toHaveBeenCalledWith('enrollment-1');
      expect(metricsService.incrementEnrollmentsRemoved).toHaveBeenCalled();
      expect(result).toBe(true);
    });

    it('should throw NotFoundException when enrollment does not exist', async () => {
      enrollmentRepository.findById.mockResolvedValue(null);

      await expect(service.removeEnrollment('non-existent-id')).rejects.toThrow(
        NotFoundException,
      );
    });

    it('should decrement section count when enrollment has section', async () => {
      const enrollmentWithSection = { ...mockEnrollment, sectionId: 'section-1' };
      enrollmentRepository.findById.mockResolvedValue(enrollmentWithSection as any);
      enrollmentRepository.delete.mockResolvedValue(true);
      sectionRepository.decrementEnrolledCount.mockResolvedValue(mockSection as any);

      await service.removeEnrollment('enrollment-1');

      expect(sectionRepository.decrementEnrolledCount).toHaveBeenCalledWith('section-1');
    });
  });

  describe('getCourseRoster', () => {
    it('should return course roster', async () => {
      const enrollments = [mockEnrollment, { ...mockEnrollment, _id: 'enrollment-2' }];

      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourse.mockResolvedValue(enrollments as any);

      const result = await service.getCourseRoster('course-1');

      expect(courseRepository.findById).toHaveBeenCalledWith('course-1');
      expect(enrollmentRepository.findByCourse).toHaveBeenCalledWith('course-1', {});
      expect(result.enrollments).toHaveLength(2);
      expect(result.totalCount).toBe(2);
    });

    it('should filter roster by section', async () => {
      const sectionEnrollments = [{ ...mockEnrollment, sectionId: 'section-1' }];

      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourse.mockResolvedValue(sectionEnrollments as any);

      await service.getCourseRoster('course-1', 'section-1');

      expect(enrollmentRepository.findByCourse).toHaveBeenCalledWith('course-1', {
        sectionId: 'section-1',
      });
    });

    it('should filter roster by status', async () => {
      const activeEnrollments = [mockEnrollment];

      courseRepository.findById.mockResolvedValue(mockCourse as any);
      enrollmentRepository.findByCourse.mockResolvedValue(activeEnrollments as any);

      await service.getCourseRoster('course-1', undefined, EnrollmentStatus.ACTIVE);

      expect(enrollmentRepository.findByCourse).toHaveBeenCalledWith('course-1', {
        status: EnrollmentStatus.ACTIVE,
      });
    });

    it('should throw NotFoundException when course does not exist', async () => {
      courseRepository.findById.mockResolvedValue(null);

      await expect(service.getCourseRoster('non-existent-course')).rejects.toThrow(
        NotFoundException,
      );
    });
  });

  describe('getStudentEnrollments', () => {
    it('should return student enrollments with course info', async () => {
      const enrollments = [mockEnrollment];

      enrollmentRepository.findByStudent.mockResolvedValue(enrollments as any);
      courseRepository.findById.mockResolvedValue(mockCourse as any);

      const result = await service.getStudentEnrollments('student-1');

      expect(enrollmentRepository.findByStudent).toHaveBeenCalledWith('student-1', {});
      expect(result).toHaveLength(1);
      expect(result[0].enrollment).toEqual(mockEnrollment);
      expect(result[0].course).toEqual(mockCourse);
    });

    it('should filter by term', async () => {
      const enrollments = [mockEnrollment];

      enrollmentRepository.findByStudent.mockResolvedValue(enrollments as any);
      courseRepository.findById.mockResolvedValue(mockCourse as any);

      await service.getStudentEnrollments('student-1', 'Fall 2024');

      expect(enrollmentRepository.findByStudent).toHaveBeenCalledWith('student-1', {
        term: 'Fall 2024',
      });
    });

    it('should filter by status', async () => {
      const enrollments = [mockEnrollment];

      enrollmentRepository.findByStudent.mockResolvedValue(enrollments as any);
      courseRepository.findById.mockResolvedValue(mockCourse as any);

      await service.getStudentEnrollments('student-1', undefined, EnrollmentStatus.ACTIVE);

      expect(enrollmentRepository.findByStudent).toHaveBeenCalledWith('student-1', {
        status: EnrollmentStatus.ACTIVE,
      });
    });

    it('should handle multiple enrollments', async () => {
      const enrollments = [
        mockEnrollment,
        { ...mockEnrollment, _id: 'enrollment-2', courseId: 'course-2' },
      ];
      const course2 = { ...mockCourse, _id: 'course-2' };

      enrollmentRepository.findByStudent.mockResolvedValue(enrollments as any);
      courseRepository.findById
        .mockResolvedValueOnce(mockCourse as any)
        .mockResolvedValueOnce(course2 as any);

      const result = await service.getStudentEnrollments('student-1');

      expect(result).toHaveLength(2);
      expect(courseRepository.findById).toHaveBeenCalledTimes(2);
    });
  });
});
