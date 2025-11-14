import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';
import { ConfigModule } from '@nestjs/config';
import { CourseModule } from '../src/course/course.module';
import { EnrollmentModule } from '../src/enrollment/enrollment.module';
import { CourseService } from '../src/course/course.service';
import { EnrollmentService } from '../src/enrollment/enrollment.service';
import configuration from '../src/config/configuration';

describe('Course Service Integration Tests', () => {
  let app: INestApplication;
  let courseService: CourseService;
  let enrollmentService: EnrollmentService;

  beforeAll(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [
        ConfigModule.forRoot({
          isGlobal: true,
          load: [configuration],
        }),
        MongooseModule.forRoot(process.env.MONGO_URI || 'mongodb://localhost:27017/course-test', {
          dbName: 'course-test',
        }),
        CourseModule,
        EnrollmentModule,
      ],
    }).compile();

    app = moduleFixture.createNestApplication();
    await app.init();

    courseService = moduleFixture.get<CourseService>(CourseService);
    enrollmentService = moduleFixture.get<EnrollmentService>(EnrollmentService);
  });

  afterAll(async () => {
    await app.close();
  });

  describe('Course Creation and Management', () => {
    let createdCourseId: string;

    it('should create a new course', async () => {
      const courseData = {
        title: 'Integration Test Course',
        description: 'This is a test course',
        term: 'Fall 2024',
        instructorId: 'instructor-test-1',
        syllabus: 'Test syllabus content',
        metadata: {
          department: 'CS',
          courseCode: 'CS999',
          credits: 3,
        },
      };

      const course = await courseService.createCourse(courseData);

      expect(course).toBeDefined();
      expect(course._id).toBeDefined();
      expect(course.title).toBe(courseData.title);
      expect(course.description).toBe(courseData.description);
      expect(course.term).toBe(courseData.term);
      expect(course.isPublished).toBe(false);

      createdCourseId = course._id.toString();
    });

    it('should retrieve the created course', async () => {
      const course = await courseService.getCourse(createdCourseId);

      expect(course).toBeDefined();
      expect(course._id.toString()).toBe(createdCourseId);
      expect(course.title).toBe('Integration Test Course');
    });

    it('should update the course', async () => {
      const updatedCourse = await courseService.updateCourse(createdCourseId, {
        title: 'Updated Integration Test Course',
        description: 'Updated description',
      });

      expect(updatedCourse).toBeDefined();
      expect(updatedCourse.title).toBe('Updated Integration Test Course');
      expect(updatedCourse.description).toBe('Updated description');
    });

    it('should list courses with filters', async () => {
      const result = await courseService.listCourses({
        instructorId: 'instructor-test-1',
        term: 'Fall 2024',
      });

      expect(result.courses).toBeDefined();
      expect(result.total).toBeGreaterThan(0);
      expect(result.courses.some((c) => c._id.toString() === createdCourseId)).toBe(true);
    });

    it('should publish the course', async () => {
      const publishedCourse = await courseService.publishCourse(createdCourseId);

      expect(publishedCourse.isPublished).toBe(true);
    });

    it('should unpublish the course', async () => {
      const unpublishedCourse = await courseService.unpublishCourse(createdCourseId);

      expect(unpublishedCourse.isPublished).toBe(false);
    });

    it('should delete the course', async () => {
      const deleted = await courseService.deleteCourse(createdCourseId);

      expect(deleted).toBe(true);

      // Verify course is deleted
      await expect(courseService.getCourse(createdCourseId)).rejects.toThrow();
    });
  });

  describe('Enrollment Workflow', () => {
    let courseId: string;
    let enrollmentId: string;

    beforeAll(async () => {
      // Create a course for enrollment tests
      const course = await courseService.createCourse({
        title: 'Enrollment Test Course',
        description: 'Test',
        term: 'Spring 2024',
        instructorId: 'instructor-enroll-1',
      });

      courseId = course._id.toString();

      // Publish the course so students can enroll
      await courseService.publishCourse(courseId);
    });

    afterAll(async () => {
      // Clean up
      try {
        await courseService.deleteCourse(courseId);
      } catch (error) {
        // Ignore if already deleted
      }
    });

    it('should allow self-enrollment', async () => {
      const enrollment = await enrollmentService.selfEnroll(courseId, 'student-test-1');

      expect(enrollment).toBeDefined();
      expect(enrollment._id).toBeDefined();
      expect(enrollment.courseId.toString()).toBe(courseId);
      expect(enrollment.studentId).toBe('student-test-1');
      expect(enrollment.enrollmentType).toBe('SELF');
      expect(enrollment.status).toBe('ACTIVE');

      enrollmentId = enrollment._id.toString();
    });

    it('should prevent duplicate enrollment', async () => {
      await expect(enrollmentService.selfEnroll(courseId, 'student-test-1')).rejects.toThrow();
    });

    it('should retrieve course roster', async () => {
      const roster = await enrollmentService.getCourseRoster(courseId);

      expect(roster.enrollments).toBeDefined();
      expect(roster.totalCount).toBe(1);
      expect(roster.enrollments[0].studentId).toBe('student-test-1');
    });

    it('should retrieve student enrollments', async () => {
      const enrollments = await enrollmentService.getStudentEnrollments('student-test-1');

      expect(enrollments).toBeDefined();
      expect(enrollments.length).toBeGreaterThan(0);
      // Check if the course matches (handle both populated and unpopulated courseId)
      expect(
        enrollments.some((e) => {
          if (!e.enrollment.courseId) return false;

          const enrollmentCourseId =
            typeof e.enrollment.courseId === 'object' && e.enrollment.courseId?._id
              ? e.enrollment.courseId._id.toString()
              : e.enrollment.courseId?.toString() || '';
          return enrollmentCourseId === courseId || e.course?._id?.toString() === courseId;
        }),
      ).toBe(true);
    });

    it('should allow instructor to add student', async () => {
      const enrollment = await enrollmentService.instructorAddStudent(
        courseId,
        'student-test-2',
        'instructor-enroll-1',
      );

      expect(enrollment).toBeDefined();
      expect(enrollment.enrollmentType).toBe('INSTRUCTOR');
      expect(enrollment.enrolledBy).toBe('instructor-enroll-1');
    });

    it('should remove enrollment', async () => {
      const removed = await enrollmentService.removeEnrollment(enrollmentId);

      expect(removed).toBe(true);

      // Verify enrollment is removed
      await expect(enrollmentService.removeEnrollment(enrollmentId)).rejects.toThrow();
    });
  });

  describe('Course Templates', () => {
    let templateId: string;

    it('should create a course template', async () => {
      const template = await courseService.createTemplate({
        name: 'Test Template',
        description: 'Template for testing',
        syllabusTemplate: 'Standard syllabus...',
        createdBy: 'admin-1',
        defaultMetadata: {
          department: 'CS',
          credits: 3,
        },
      });

      expect(template).toBeDefined();
      expect(template._id).toBeDefined();
      expect(template.name).toBe('Test Template');

      templateId = template._id.toString();
    });

    it('should create course from template', async () => {
      const course = await courseService.createCourseFromTemplate(templateId, {
        title: 'Course from Template',
        term: 'Fall 2024',
        instructorId: 'instructor-template-1',
      });

      expect(course).toBeDefined();
      expect(course.title).toBe('Course from Template');
      expect(course.templateId.toString()).toBe(templateId);
      expect(course.syllabus).toBe('Standard syllabus...');

      // Clean up
      await courseService.deleteCourse(course._id.toString());
    });
  });

  describe('Prerequisites', () => {
    let course1Id: string;
    let course2Id: string;

    beforeAll(async () => {
      const course1 = await courseService.createCourse({
        title: 'Prerequisite Course 1',
        description: 'First course',
        term: 'Fall 2024',
        instructorId: 'instructor-prereq-1',
      });

      const course2 = await courseService.createCourse({
        title: 'Advanced Course',
        description: 'Advanced course',
        term: 'Spring 2025',
        instructorId: 'instructor-prereq-1',
      });

      course1Id = course1._id.toString();
      course2Id = course2._id.toString();
    });

    afterAll(async () => {
      try {
        await courseService.deleteCourse(course1Id);
        await courseService.deleteCourse(course2Id);
      } catch (error) {
        // Ignore errors
      }
    });

    it('should add prerequisite to course', async () => {
      const course = await courseService.addPrerequisite(course2Id, course1Id);

      expect(course.prerequisiteCourseIds).toContain(course1Id);
    });

    it('should prevent circular prerequisites', async () => {
      await expect(courseService.addPrerequisite(course1Id, course2Id)).rejects.toThrow(
        'Circular prerequisite dependency detected',
      );
    });

    it('should check prerequisites', async () => {
      const result = await courseService.checkPrerequisites(course2Id, 'student-prereq-1');

      expect(result.hasPrerequisites).toBe(false);
      expect(result.missingPrerequisiteIds).toContain(course1Id);
    });

    it('should remove prerequisite', async () => {
      const course = await courseService.removePrerequisite(course2Id, course1Id);

      expect(course.prerequisiteCourseIds).not.toContain(course1Id);
    });
  });

  describe('Co-teaching', () => {
    let courseId: string;

    beforeAll(async () => {
      const course = await courseService.createCourse({
        title: 'Co-taught Course',
        description: 'Course with multiple instructors',
        term: 'Fall 2024',
        instructorId: 'instructor-main-1',
      });

      courseId = course._id.toString();
    });

    afterAll(async () => {
      try {
        await courseService.deleteCourse(courseId);
      } catch (error) {
        // Ignore
      }
    });

    it('should add co-instructor', async () => {
      const course = await courseService.addCoInstructor(courseId, 'co-instructor-1');

      expect(course.coInstructorIds).toContain('co-instructor-1');
    });

    it('should remove co-instructor', async () => {
      const course = await courseService.removeCoInstructor(courseId, 'co-instructor-1');

      expect(course.coInstructorIds).not.toContain('co-instructor-1');
    });
  });

  describe('Sections', () => {
    let courseId: string;
    let sectionId: string;

    beforeAll(async () => {
      const course = await courseService.createCourse({
        title: 'Course with Sections',
        description: 'Test sections',
        term: 'Fall 2024',
        instructorId: 'instructor-section-1',
      });

      courseId = course._id.toString();
    });

    afterAll(async () => {
      try {
        await courseService.deleteCourse(courseId);
      } catch (error) {
        // Ignore
      }
    });

    it('should create a section', async () => {
      const section = await courseService.createSection({
        courseId,
        sectionNumber: '001',
        instructorId: 'instructor-section-1',
        schedule: {
          daysOfWeek: ['MON', 'WED', 'FRI'],
          startTime: '09:00',
          endTime: '10:30',
        },
        location: 'Room 101',
        maxStudents: 30,
      });

      expect(section).toBeDefined();
      expect(section._id).toBeDefined();
      expect(section.sectionNumber).toBe('001');

      sectionId = section._id.toString();
    });

    it('should list course sections', async () => {
      const sections = await courseService.listCourseSections(courseId);

      expect(sections).toBeDefined();
      expect(sections.length).toBe(1);
      expect(sections[0]._id.toString()).toBe(sectionId);
    });

    it('should update section', async () => {
      const updated = await courseService.updateSection(sectionId, {
        maxStudents: 40,
        location: 'Room 202',
      });

      expect(updated.maxStudents).toBe(40);
      expect(updated.location).toBe('Room 202');
    });

    it('should delete section', async () => {
      const deleted = await courseService.deleteSection(sectionId);

      expect(deleted).toBe(true);
    });
  });
});
