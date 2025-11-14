/**
 * Test utilities and helper functions for Course Management Service
 */

import { EnrollmentType, EnrollmentStatus } from '../src/enrollment/schemas/enrollment.schema';

/**
 * Create a mock course object
 */
export const createMockCourse = (overrides: any = {}) => ({
  _id: 'course-id',
  title: 'Test Course',
  description: 'Test Description',
  term: 'Fall 2024',
  syllabus: 'Test Syllabus',
  instructorId: 'instructor-id',
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
  ...overrides,
});

/**
 * Create a mock enrollment object
 */
export const createMockEnrollment = (overrides: any = {}) => ({
  _id: 'enrollment-id',
  courseId: 'course-id',
  studentId: 'student-id',
  enrollmentType: EnrollmentType.SELF,
  status: EnrollmentStatus.ACTIVE,
  enrolledBy: 'student-id',
  enrolledAt: new Date(),
  sectionId: null,
  ...overrides,
});

/**
 * Create a mock course template object
 */
export const createMockTemplate = (overrides: any = {}) => ({
  _id: 'template-id',
  name: 'Test Template',
  description: 'Test Template Description',
  syllabusTemplate: 'Template Syllabus',
  createdBy: 'admin-id',
  defaultMetadata: {
    department: 'CS',
    credits: 3,
  },
  createdAt: new Date(),
  ...overrides,
});

/**
 * Create a mock section object
 */
export const createMockSection = (overrides: any = {}) => ({
  _id: 'section-id',
  courseId: 'course-id',
  sectionNumber: '001',
  instructorId: 'instructor-id',
  schedule: {
    daysOfWeek: ['MON', 'WED', 'FRI'],
    startTime: '09:00',
    endTime: '10:30',
  },
  location: 'Room 101',
  maxStudents: 30,
  enrolledCount: 0,
  createdAt: new Date(),
  ...overrides,
});

/**
 * Create a mock Mongoose model
 */
export const createMockMongooseModel = () => ({
  constructor: jest.fn(),
  find: jest.fn(),
  findById: jest.fn(),
  findOne: jest.fn(),
  findByIdAndUpdate: jest.fn(),
  findByIdAndDelete: jest.fn(),
  findOneAndUpdate: jest.fn(),
  findOneAndDelete: jest.fn(),
  countDocuments: jest.fn(),
  create: jest.fn(),
  save: jest.fn(),
  exec: jest.fn(),
});

/**
 * Wait for a specified duration (useful for timing tests)
 */
export const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Mock execution context for testing interceptors
 */
export const createMockExecutionContext = (methodName: string = 'testMethod') => ({
  switchToRpc: jest.fn().mockReturnValue({
    getContext: jest.fn(),
    getData: jest.fn(),
  }),
  switchToHttp: jest.fn(),
  getHandler: jest.fn().mockReturnValue({ name: methodName }),
  getClass: jest.fn(),
  getArgs: jest.fn(),
  getArgByIndex: jest.fn(),
  getType: jest.fn(),
});
