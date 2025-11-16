import { Test, TestingModule } from '@nestjs/testing';
import { getModelToken } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { CourseRepository } from './course.repository';
import { Course } from '../schemas/course.schema';

describe('CourseRepository', () => {
  let repository: CourseRepository;
  let model: jest.Mocked<Model<Course>>;

  const mockCourse = {
    _id: 'course-1',
    title: 'Test Course',
    description: 'Test Description',
    term: 'Fall 2024',
    instructorId: 'instructor-1',
    coInstructorIds: [],
    isPublished: false,
    prerequisiteCourseIds: [],
    save: jest.fn().mockResolvedValue(this),
  };

  beforeEach(async () => {
    const mockModel = {
      constructor: jest.fn(),
      create: jest.fn(),
      findById: jest.fn(),
      find: jest.fn(),
      findByIdAndUpdate: jest.fn(),
      findByIdAndDelete: jest.fn(),
      findOneAndUpdate: jest.fn(),
      countDocuments: jest.fn(),
      exec: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        CourseRepository,
        {
          provide: getModelToken(Course.name),
          useValue: mockModel,
        },
      ],
    }).compile();

    repository = module.get<CourseRepository>(CourseRepository);
    model = module.get(getModelToken(Course.name));
  });

  describe('findById', () => {
    it('should find course by id', async () => {
      const execMock = jest.fn().mockResolvedValue(mockCourse);
      (model.findById as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.findById('course-1');

      expect(model.findById).toHaveBeenCalledWith('course-1');
      expect(result).toEqual(mockCourse);
    });

    it('should return null when course not found', async () => {
      const execMock = jest.fn().mockResolvedValue(null);
      (model.findById as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.findById('non-existent');

      expect(result).toBeNull();
    });
  });

  describe('findAll', () => {
    it('should find courses with filters', async () => {
      const courses = [mockCourse];
      const execMock = jest.fn().mockResolvedValue(courses);
      const countMock = jest.fn().mockResolvedValue(1);

      (model.find as jest.Mock).mockReturnValue({
        skip: jest.fn().mockReturnThis(),
        limit: jest.fn().mockReturnThis(),
        exec: execMock,
      });
      (model.countDocuments as jest.Mock).mockReturnValue({ exec: countMock });

      const result = await repository.findAll({
        instructorId: 'instructor-1',
        page: 1,
        pageSize: 20,
      });

      expect(result.courses).toEqual(courses);
      expect(result.total).toBe(1);
    });
  });

  describe('update', () => {
    it('should update course', async () => {
      const updated = { ...mockCourse, title: 'Updated Title' };
      const execMock = jest.fn().mockResolvedValue(updated);
      (model.findByIdAndUpdate as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.update('course-1', { title: 'Updated Title' });

      expect(model.findByIdAndUpdate).toHaveBeenCalledWith(
        'course-1',
        { title: 'Updated Title' },
        { new: true },
      );
      expect(result).toEqual(updated);
    });
  });

  describe('delete', () => {
    it('should delete course and return true', async () => {
      const execMock = jest.fn().mockResolvedValue(mockCourse);
      (model.findByIdAndDelete as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.delete('course-1');

      expect(model.findByIdAndDelete).toHaveBeenCalledWith('course-1');
      expect(result).toBe(true);
    });

    it('should return false when course not found', async () => {
      const execMock = jest.fn().mockResolvedValue(null);
      (model.findByIdAndDelete as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.delete('non-existent');

      expect(result).toBe(false);
    });
  });

  describe('publish/unpublish', () => {
    it('should publish course', async () => {
      const published = { ...mockCourse, isPublished: true };
      const execMock = jest.fn().mockResolvedValue(published);
      (model.findByIdAndUpdate as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.publish('course-1');

      expect(model.findByIdAndUpdate).toHaveBeenCalledWith(
        'course-1',
        { isPublished: true },
        { new: true },
      );
      expect(result?.isPublished).toBe(true);
    });

    it('should unpublish course', async () => {
      const unpublished = { ...mockCourse, isPublished: false };
      const execMock = jest.fn().mockResolvedValue(unpublished);
      (model.findByIdAndUpdate as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.unpublish('course-1');

      expect(result?.isPublished).toBe(false);
    });
  });

  describe('addPrerequisite', () => {
    it('should add prerequisite to course', async () => {
      const withPrereq = { ...mockCourse, prerequisiteCourseIds: ['prereq-1'] };
      const execMock = jest.fn().mockResolvedValue(withPrereq);
      (model.findByIdAndUpdate as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.addPrerequisite('course-1', 'prereq-1');

      expect(model.findByIdAndUpdate).toHaveBeenCalledWith(
        'course-1',
        { $addToSet: { prerequisiteCourseIds: 'prereq-1' } },
        { new: true },
      );
      expect(result?.prerequisiteCourseIds).toContain('prereq-1');
    });
  });

  describe('addCoInstructor', () => {
    it('should add co-instructor to course', async () => {
      const withCoInstructor = { ...mockCourse, coInstructorIds: ['co-instructor-1'] };
      const execMock = jest.fn().mockResolvedValue(withCoInstructor);
      (model.findByIdAndUpdate as jest.Mock).mockReturnValue({ exec: execMock });

      const result = await repository.addCoInstructor('course-1', 'co-instructor-1');

      expect(model.findByIdAndUpdate).toHaveBeenCalledWith(
        'course-1',
        { $addToSet: { coInstructorIds: 'co-instructor-1' } },
        { new: true },
      );
      expect(result?.coInstructorIds).toContain('co-instructor-1');
    });
  });
});
