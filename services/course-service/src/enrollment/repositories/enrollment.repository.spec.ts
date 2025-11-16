import { Test, TestingModule } from '@nestjs/testing';
import { getModelToken } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { EnrollmentRepository } from './enrollment.repository';
import { Enrollment, EnrollmentStatus, EnrollmentType } from '../schemas/enrollment.schema';

describe('EnrollmentRepository', () => {
  let repository: EnrollmentRepository;
  let model: jest.Mocked<Model<Enrollment>>;

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
    const mockModel = {
      findOne: jest.fn(),
      find: jest.fn(),
      findById: jest.fn(),
      findByIdAndUpdate: jest.fn(),
      findByIdAndDelete: jest.fn(),
      countDocuments: jest.fn(),
      exec: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        EnrollmentRepository,
        {
          provide: getModelToken(Enrollment.name),
          useValue: mockModel,
        },
      ],
    }).compile();

    repository = module.get<EnrollmentRepository>(EnrollmentRepository);
    model = module.get(getModelToken(Enrollment.name));
  });

  describe('findByCourseAndStudent', () => {
    it('should find enrollment by course and student', async () => {
      const execMock = jest.fn().mockResolvedValue(mockEnrollment);
      (model.findOne as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.findByCourseAndStudent('course-1', 'student-1');

      expect(model.findOne).toHaveBeenCalledWith({
        courseId: 'course-1',
        studentId: 'student-1',
      });
      expect(result).toEqual(mockEnrollment);
    });

    it('should return null when enrollment not found', async () => {
      const execMock = jest.fn().mockResolvedValue(null);
      (model.findOne as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.findByCourseAndStudent('course-1', 'student-2');

      expect(result).toBeNull();
    });
  });

  describe('findByCourse', () => {
    it('should find enrollments by course', async () => {
      const enrollments = [mockEnrollment];
      const execMock = jest.fn().mockResolvedValue(enrollments);
      (model.find as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.findByCourse('course-1');

      expect(model.find).toHaveBeenCalledWith({ courseId: 'course-1' });
      expect(result).toEqual(enrollments);
    });

    it('should filter by section', async () => {
      const execMock = jest.fn().mockResolvedValue([mockEnrollment]);
      (model.find as jest.Mock).mockReturnValue({ exec: execMock });

      await repository.findByCourse('course-1', { sectionId: 'section-1' });

      expect(model.find).toHaveBeenCalledWith({
        courseId: 'course-1',
        sectionId: 'section-1',
      });
    });

    it('should filter by status', async () => {
      const execMock = jest.fn().mockResolvedValue([mockEnrollment]);
      (model.find as jest.Mock).mockReturnValue({ exec: execMock });

      await repository.findByCourse('course-1', { status: EnrollmentStatus.ACTIVE });

      expect(model.find).toHaveBeenCalledWith({
        courseId: 'course-1',
        status: EnrollmentStatus.ACTIVE,
      });
    });
  });

  describe('delete', () => {
    it('should delete enrollment and return true', async () => {
      const execMock = jest.fn().mockResolvedValue(mockEnrollment);
      (model.findByIdAndDelete as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.delete('enrollment-1');

      expect(model.findByIdAndDelete).toHaveBeenCalledWith('enrollment-1');
      expect(result).toBe(true);
    });

    it('should return false when enrollment not found', async () => {
      const execMock = jest.fn().mockResolvedValue(null);
      (model.findByIdAndDelete as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.delete('non-existent');

      expect(result).toBe(false);
    });
  });

  describe('countByCourse', () => {
    it('should count enrollments by course', async () => {
      const execMock = jest.fn().mockResolvedValue(5);
      (model.countDocuments as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.countByCourse('course-1');

      expect(model.countDocuments).toHaveBeenCalledWith({ courseId: 'course-1' });
      expect(result).toBe(5);
    });

    it('should count by course and status', async () => {
      const execMock = jest.fn().mockResolvedValue(3);
      (model.countDocuments as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.countByCourse('course-1', EnrollmentStatus.ACTIVE);

      expect(model.countDocuments).toHaveBeenCalledWith({
        courseId: 'course-1',
        status: EnrollmentStatus.ACTIVE,
      });
      expect(result).toBe(3);
    });
  });
});
