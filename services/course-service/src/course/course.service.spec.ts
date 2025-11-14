import { Test, TestingModule } from '@nestjs/testing';
import { NotFoundException, BadRequestException } from '@nestjs/common';
import { CourseService } from './course.service';
import { CourseRepository } from './repositories/course.repository';
import { CourseTemplateRepository } from './repositories/course-template.repository';
import { SectionRepository } from './repositories/section.repository';
import { CrossListingRepository } from './repositories/cross-listing.repository';
import { MetricsService } from '../observability/metrics.service';

describe('CourseService', () => {
  let service: CourseService;
  let courseRepository: jest.Mocked<CourseRepository>;
  let templateRepository: jest.Mocked<CourseTemplateRepository>;
  let sectionRepository: jest.Mocked<SectionRepository>;
  let crossListingRepository: jest.Mocked<CrossListingRepository>;
  let metricsService: jest.Mocked<MetricsService>;

  const mockCourse = {
    _id: 'course-id-1',
    title: 'Introduction to Computer Science',
    description: 'Learn the basics of CS',
    term: 'Fall 2024',
    syllabus: 'Course syllabus...',
    instructorId: 'instructor-1',
    coInstructorIds: [],
    isPublished: false,
    prerequisiteCourseIds: [],
    metadata: {
      department: 'CS',
      courseCode: 'CS101',
      credits: 3,
    },
    createdAt: new Date(),
    updatedAt: new Date(),
  };

  const mockTemplate = {
    _id: 'template-1',
    name: 'Standard CS Course',
    description: 'Template for CS courses',
    syllabusTemplate: 'Standard syllabus...',
    createdBy: 'admin-1',
    defaultMetadata: {
      department: 'CS',
      credits: 3,
    },
    createdAt: new Date(),
  };

  beforeEach(async () => {
    const mockCourseRepo = {
      create: jest.fn(),
      findById: jest.fn(),
      findAll: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
      publish: jest.fn(),
      unpublish: jest.fn(),
      addPrerequisite: jest.fn(),
      removePrerequisite: jest.fn(),
      addCoInstructor: jest.fn(),
      removeCoInstructor: jest.fn(),
      findByCrossListingGroup: jest.fn(),
      findByIds: jest.fn(),
    };

    const mockTemplateRepo = {
      create: jest.fn(),
      findById: jest.fn(),
      findAll: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
    };

    const mockSectionRepo = {
      create: jest.fn(),
      findById: jest.fn(),
      findByCourse: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
    };

    const mockCrossListingRepo = {
      create: jest.fn(),
      findByGroupId: jest.fn(),
      findByCourseId: jest.fn(),
      delete: jest.fn(),
    };

    const mockMetricsService = {
      incrementCoursesCreated: jest.fn(),
      incrementCoursesPublished: jest.fn(),
      incrementCoursesUnpublished: jest.fn(),
      incrementTemplatesCreated: jest.fn(),
      incrementCoursesFromTemplates: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        CourseService,
        { provide: CourseRepository, useValue: mockCourseRepo },
        { provide: CourseTemplateRepository, useValue: mockTemplateRepo },
        { provide: SectionRepository, useValue: mockSectionRepo },
        { provide: CrossListingRepository, useValue: mockCrossListingRepo },
        { provide: MetricsService, useValue: mockMetricsService },
      ],
    }).compile();

    service = module.get<CourseService>(CourseService);
    courseRepository = module.get(CourseRepository);
    templateRepository = module.get(CourseTemplateRepository);
    sectionRepository = module.get(SectionRepository);
    crossListingRepository = module.get(CrossListingRepository);
    metricsService = module.get(MetricsService);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('createCourse', () => {
    it('should create a course successfully', async () => {
      courseRepository.create.mockResolvedValue(mockCourse as any);

      const result = await service.createCourse({
        title: mockCourse.title,
        description: mockCourse.description,
        term: mockCourse.term,
        instructorId: mockCourse.instructorId,
      });

      expect(courseRepository.create).toHaveBeenCalledWith({
        title: mockCourse.title,
        description: mockCourse.description,
        term: mockCourse.term,
        instructorId: mockCourse.instructorId,
      });
      expect(metricsService.incrementCoursesCreated).toHaveBeenCalledWith(true);
      expect(result).toEqual(mockCourse);
    });

    it('should track failed course creation in metrics', async () => {
      const error = new Error('Database error');
      courseRepository.create.mockRejectedValue(error);

      await expect(
        service.createCourse({
          title: 'Test Course',
          description: 'Test',
          term: 'Fall 2024',
          instructorId: 'instructor-1',
        }),
      ).rejects.toThrow(error);

      expect(metricsService.incrementCoursesCreated).toHaveBeenCalledWith(false);
    });
  });

  describe('getCourse', () => {
    it('should return a course when found', async () => {
      courseRepository.findById.mockResolvedValue(mockCourse as any);

      const result = await service.getCourse('course-id-1');

      expect(courseRepository.findById).toHaveBeenCalledWith('course-id-1');
      expect(result).toEqual(mockCourse);
    });

    it('should throw NotFoundException when course not found', async () => {
      courseRepository.findById.mockResolvedValue(null);

      await expect(service.getCourse('non-existent-id')).rejects.toThrow(NotFoundException);
      await expect(service.getCourse('non-existent-id')).rejects.toThrow(
        'Course not found: non-existent-id',
      );
    });
  });

  describe('listCourses', () => {
    it('should list courses with filters', async () => {
      const mockCourses = [mockCourse];
      courseRepository.findAll.mockResolvedValue({ courses: mockCourses as any, total: 1 });

      const result = await service.listCourses({
        instructorId: 'instructor-1',
        term: 'Fall 2024',
        page: 1,
        pageSize: 20,
      });

      expect(courseRepository.findAll).toHaveBeenCalledWith({
        instructorId: 'instructor-1',
        term: 'Fall 2024',
        page: 1,
        pageSize: 20,
      });
      expect(result).toEqual({ courses: mockCourses, total: 1 });
    });
  });

  describe('updateCourse', () => {
    it('should update a course successfully', async () => {
      const updatedCourse = { ...mockCourse, title: 'Updated Title' };
      courseRepository.update.mockResolvedValue(updatedCourse as any);

      const result = await service.updateCourse('course-id-1', { title: 'Updated Title' });

      expect(courseRepository.update).toHaveBeenCalledWith('course-id-1', {
        title: 'Updated Title',
      });
      expect(result).toEqual(updatedCourse);
    });

    it('should throw NotFoundException when updating non-existent course', async () => {
      courseRepository.update.mockResolvedValue(null);

      await expect(service.updateCourse('non-existent-id', { title: 'New Title' })).rejects.toThrow(
        NotFoundException,
      );
    });
  });

  describe('deleteCourse', () => {
    it('should delete a course successfully', async () => {
      courseRepository.delete.mockResolvedValue(true);

      const result = await service.deleteCourse('course-id-1');

      expect(courseRepository.delete).toHaveBeenCalledWith('course-id-1');
      expect(result).toBe(true);
    });

    it('should throw NotFoundException when deleting non-existent course', async () => {
      courseRepository.delete.mockResolvedValue(false);

      await expect(service.deleteCourse('non-existent-id')).rejects.toThrow(NotFoundException);
    });
  });

  describe('publishCourse', () => {
    it('should publish a course successfully', async () => {
      const publishedCourse = { ...mockCourse, isPublished: true };
      courseRepository.publish.mockResolvedValue(publishedCourse as any);

      const result = await service.publishCourse('course-id-1');

      expect(courseRepository.publish).toHaveBeenCalledWith('course-id-1');
      expect(metricsService.incrementCoursesPublished).toHaveBeenCalled();
      expect(result.isPublished).toBe(true);
    });

    it('should throw NotFoundException when publishing non-existent course', async () => {
      courseRepository.publish.mockResolvedValue(null);

      await expect(service.publishCourse('non-existent-id')).rejects.toThrow(NotFoundException);
      expect(metricsService.incrementCoursesPublished).not.toHaveBeenCalled();
    });
  });

  describe('unpublishCourse', () => {
    it('should unpublish a course successfully', async () => {
      const unpublishedCourse = { ...mockCourse, isPublished: false };
      courseRepository.unpublish.mockResolvedValue(unpublishedCourse as any);

      const result = await service.unpublishCourse('course-id-1');

      expect(courseRepository.unpublish).toHaveBeenCalledWith('course-id-1');
      expect(metricsService.incrementCoursesUnpublished).toHaveBeenCalled();
      expect(result.isPublished).toBe(false);
    });
  });

  describe('Template Operations', () => {
    describe('createTemplate', () => {
      it('should create a template successfully', async () => {
        templateRepository.create.mockResolvedValue(mockTemplate as any);

        const result = await service.createTemplate({
          name: mockTemplate.name,
          description: mockTemplate.description,
          createdBy: mockTemplate.createdBy,
        });

        expect(templateRepository.create).toHaveBeenCalled();
        expect(metricsService.incrementTemplatesCreated).toHaveBeenCalled();
        expect(result).toEqual(mockTemplate);
      });
    });

    describe('getTemplate', () => {
      it('should return a template when found', async () => {
        templateRepository.findById.mockResolvedValue(mockTemplate as any);

        const result = await service.getTemplate('template-1');

        expect(templateRepository.findById).toHaveBeenCalledWith('template-1');
        expect(result).toEqual(mockTemplate);
      });

      it('should throw NotFoundException when template not found', async () => {
        templateRepository.findById.mockResolvedValue(null);

        await expect(service.getTemplate('non-existent-id')).rejects.toThrow(NotFoundException);
      });
    });

    describe('createCourseFromTemplate', () => {
      it('should create a course from template successfully', async () => {
        templateRepository.findById.mockResolvedValue(mockTemplate as any);
        courseRepository.create.mockResolvedValue(mockCourse as any);

        const result = await service.createCourseFromTemplate('template-1', {
          title: 'New Course from Template',
          term: 'Fall 2024',
          instructorId: 'instructor-1',
        });

        expect(templateRepository.findById).toHaveBeenCalledWith('template-1');
        expect(courseRepository.create).toHaveBeenCalledWith(
          expect.objectContaining({
            title: 'New Course from Template',
            term: 'Fall 2024',
            instructorId: 'instructor-1',
            templateId: mockTemplate._id,
          }),
        );
        expect(metricsService.incrementCoursesFromTemplates).toHaveBeenCalled();
        expect(result).toEqual(mockCourse);
      });
    });
  });

  describe('Prerequisite Operations', () => {
    describe('addPrerequisite', () => {
      it('should add a prerequisite successfully', async () => {
        const prerequisiteCourse = { ...mockCourse, _id: 'prereq-1' };
        const updatedCourse = {
          ...mockCourse,
          prerequisiteCourseIds: ['prereq-1'],
        };

        courseRepository.findById
          .mockResolvedValueOnce(mockCourse as any)
          .mockResolvedValueOnce(prerequisiteCourse as any);
        courseRepository.addPrerequisite.mockResolvedValue(updatedCourse as any);

        const result = await service.addPrerequisite('course-id-1', 'prereq-1');

        expect(courseRepository.addPrerequisite).toHaveBeenCalledWith('course-id-1', 'prereq-1');
        expect(result.prerequisiteCourseIds).toContain('prereq-1');
      });

      it('should detect circular prerequisite dependencies', async () => {
        const course1 = { ...mockCourse, _id: 'course-1', prerequisiteCourseIds: [] };
        const course2 = {
          ...mockCourse,
          _id: 'course-2',
          prerequisiteCourseIds: ['course-1'],
        };

        courseRepository.findById
          .mockResolvedValueOnce(course1 as any)
          .mockResolvedValueOnce(course2 as any)
          .mockResolvedValueOnce(course2 as any)
          .mockResolvedValueOnce(course1 as any);

        await expect(service.addPrerequisite('course-1', 'course-2')).rejects.toThrow(
          'Circular prerequisite dependency detected',
        );
      });
    });

    describe('removePrerequisite', () => {
      it('should remove a prerequisite successfully', async () => {
        const updatedCourse = { ...mockCourse, prerequisiteCourseIds: [] };
        courseRepository.removePrerequisite.mockResolvedValue(updatedCourse as any);

        const result = await service.removePrerequisite('course-id-1', 'prereq-1');

        expect(courseRepository.removePrerequisite).toHaveBeenCalledWith('course-id-1', 'prereq-1');
        expect(result).toEqual(updatedCourse);
      });
    });

    describe('checkPrerequisites', () => {
      it('should return true when course has no prerequisites', async () => {
        courseRepository.findById.mockResolvedValue(mockCourse as any);

        const result = await service.checkPrerequisites('course-id-1', 'student-1');

        expect(result.hasPrerequisites).toBe(true);
        expect(result.missingPrerequisiteIds).toHaveLength(0);
      });

      it('should return missing prerequisites', async () => {
        const courseWithPrereqs = {
          ...mockCourse,
          prerequisiteCourseIds: ['prereq-1', 'prereq-2'],
        };
        const prereqs = [
          { ...mockCourse, _id: 'prereq-1' },
          { ...mockCourse, _id: 'prereq-2' },
        ];

        courseRepository.findById.mockResolvedValue(courseWithPrereqs as any);
        courseRepository.findByIds.mockResolvedValue(prereqs as any);

        const result = await service.checkPrerequisites('course-id-1', 'student-1');

        expect(result.hasPrerequisites).toBe(false);
        expect(result.missingPrerequisiteIds).toHaveLength(2);
      });
    });
  });

  describe('Co-Instructor Operations', () => {
    describe('addCoInstructor', () => {
      it('should add a co-instructor successfully', async () => {
        const updatedCourse = {
          ...mockCourse,
          coInstructorIds: ['co-instructor-1'],
        };
        courseRepository.addCoInstructor.mockResolvedValue(updatedCourse as any);

        const result = await service.addCoInstructor('course-id-1', 'co-instructor-1');

        expect(courseRepository.addCoInstructor).toHaveBeenCalledWith(
          'course-id-1',
          'co-instructor-1',
        );
        expect(result.coInstructorIds).toContain('co-instructor-1');
      });

      it('should throw NotFoundException when course not found', async () => {
        courseRepository.addCoInstructor.mockResolvedValue(null);

        await expect(service.addCoInstructor('non-existent-id', 'co-instructor-1')).rejects.toThrow(
          NotFoundException,
        );
      });
    });

    describe('removeCoInstructor', () => {
      it('should remove a co-instructor successfully', async () => {
        const updatedCourse = { ...mockCourse, coInstructorIds: [] };
        courseRepository.removeCoInstructor.mockResolvedValue(updatedCourse as any);

        const result = await service.removeCoInstructor('course-id-1', 'co-instructor-1');

        expect(courseRepository.removeCoInstructor).toHaveBeenCalledWith(
          'course-id-1',
          'co-instructor-1',
        );
        expect(result).toEqual(updatedCourse);
      });
    });
  });

  describe('Section Operations', () => {
    const mockSection = {
      _id: 'section-1',
      courseId: 'course-id-1',
      sectionNumber: '001',
      instructorId: 'instructor-1',
      schedule: { daysOfWeek: ['MON', 'WED'], startTime: '09:00', endTime: '10:30' },
      location: 'Room 101',
      maxStudents: 30,
      enrolledCount: 0,
    };

    describe('createSection', () => {
      it('should create a section successfully', async () => {
        courseRepository.findById.mockResolvedValue(mockCourse as any);
        sectionRepository.create.mockResolvedValue(mockSection as any);

        const result = await service.createSection({
          courseId: 'course-id-1',
          sectionNumber: '001',
          instructorId: 'instructor-1',
          schedule: mockSection.schedule,
        });

        expect(courseRepository.findById).toHaveBeenCalledWith('course-id-1');
        expect(sectionRepository.create).toHaveBeenCalled();
        expect(result).toEqual(mockSection);
      });

      it('should throw NotFoundException when course not found', async () => {
        courseRepository.findById.mockResolvedValue(null);

        await expect(
          service.createSection({
            courseId: 'non-existent-id',
            sectionNumber: '001',
            instructorId: 'instructor-1',
            schedule: mockSection.schedule,
          }),
        ).rejects.toThrow(NotFoundException);
      });
    });

    describe('listCourseSections', () => {
      it('should list sections for a course', async () => {
        courseRepository.findById.mockResolvedValue(mockCourse as any);
        sectionRepository.findByCourse.mockResolvedValue([mockSection] as any);

        const result = await service.listCourseSections('course-id-1');

        expect(sectionRepository.findByCourse).toHaveBeenCalledWith('course-id-1');
        expect(result).toHaveLength(1);
        expect(result[0]).toEqual(mockSection);
      });
    });
  });

  describe('Cross-Listing Operations', () => {
    describe('crossListCourses', () => {
      it('should create a cross-listing successfully', async () => {
        const course1 = { ...mockCourse, _id: 'course-1' };
        const course2 = { ...mockCourse, _id: 'course-2' };

        courseRepository.findById
          .mockResolvedValueOnce(course1 as any)
          .mockResolvedValueOnce(course2 as any);

        const mockCrossListing = {
          _id: 'cross-listing-1',
          groupId: expect.any(String),
          courseIds: ['course-1', 'course-2'],
          createdBy: 'admin-1',
        };

        crossListingRepository.create.mockResolvedValue(mockCrossListing as any);
        courseRepository.update.mockResolvedValue(course1 as any);

        const result = await service.crossListCourses(['course-1', 'course-2'], 'admin-1');

        expect(crossListingRepository.create).toHaveBeenCalled();
        expect(courseRepository.update).toHaveBeenCalledTimes(2);
        expect(result.courseIds).toEqual(['course-1', 'course-2']);
      });
    });

    describe('getCrossListedCourses', () => {
      it('should return all cross-listed courses', async () => {
        const courseWithGroup = { ...mockCourse, crossListingGroupId: 'group-1' };
        const crossListedCourses = [
          { ...mockCourse, _id: 'course-1', crossListingGroupId: 'group-1' },
          { ...mockCourse, _id: 'course-2', crossListingGroupId: 'group-1' },
        ];

        courseRepository.findById.mockResolvedValue(courseWithGroup as any);
        courseRepository.findByCrossListingGroup.mockResolvedValue(crossListedCourses as any);

        const result = await service.getCrossListedCourses('course-id-1');

        expect(result.groupId).toBe('group-1');
        expect(result.courses).toHaveLength(2);
      });

      it('should return single course when not cross-listed', async () => {
        courseRepository.findById.mockResolvedValue(mockCourse as any);

        const result = await service.getCrossListedCourses('course-id-1');

        expect(result.groupId).toBeNull();
        expect(result.courses).toHaveLength(1);
        expect(result.courses[0]).toEqual(mockCourse);
      });
    });
  });
});
