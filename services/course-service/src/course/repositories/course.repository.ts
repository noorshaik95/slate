import { Injectable } from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { Course, CourseDocument } from '../schemas/course.schema';

@Injectable()
export class CourseRepository {
  constructor(@InjectModel(Course.name) private courseModel: Model<CourseDocument>) {}

  async create(course: Partial<Course>): Promise<CourseDocument> {
    const createdCourse = new this.courseModel(course);
    return createdCourse.save();
  }

  async findById(id: string): Promise<CourseDocument | null> {
    return this.courseModel.findById(id).exec();
  }

  async findAll(filters: {
    instructorId?: string;
    term?: string;
    isPublished?: boolean;
    page?: number;
    pageSize?: number;
  }): Promise<{ courses: CourseDocument[]; total: number }> {
    const query: any = {};

    if (filters.instructorId) {
      query.$or = [
        { instructorId: filters.instructorId },
        { coInstructorIds: filters.instructorId },
      ];
    }

    if (filters.term) {
      query.term = filters.term;
    }

    if (filters.isPublished !== undefined) {
      query.isPublished = filters.isPublished;
    }

    const page = filters.page || 1;
    const pageSize = filters.pageSize || 20;
    const skip = (page - 1) * pageSize;

    const [courses, total] = await Promise.all([
      this.courseModel.find(query).skip(skip).limit(pageSize).exec(),
      this.courseModel.countDocuments(query).exec(),
    ]);

    return { courses, total };
  }

  async update(id: string, updates: Partial<Course>): Promise<CourseDocument | null> {
    return this.courseModel.findByIdAndUpdate(id, updates, { new: true }).exec();
  }

  async delete(id: string): Promise<boolean> {
    const result = await this.courseModel.findByIdAndDelete(id).exec();
    return result !== null;
  }

  async addPrerequisite(courseId: string, prerequisiteId: string): Promise<CourseDocument | null> {
    return this.courseModel
      .findByIdAndUpdate(
        courseId,
        { $addToSet: { prerequisiteCourseIds: prerequisiteId } },
        { new: true },
      )
      .exec();
  }

  async removePrerequisite(
    courseId: string,
    prerequisiteId: string,
  ): Promise<CourseDocument | null> {
    return this.courseModel
      .findByIdAndUpdate(
        courseId,
        { $pull: { prerequisiteCourseIds: prerequisiteId } },
        { new: true },
      )
      .exec();
  }

  async addCoInstructor(courseId: string, coInstructorId: string): Promise<CourseDocument | null> {
    return this.courseModel
      .findByIdAndUpdate(
        courseId,
        { $addToSet: { coInstructorIds: coInstructorId } },
        { new: true },
      )
      .exec();
  }

  async removeCoInstructor(
    courseId: string,
    coInstructorId: string,
  ): Promise<CourseDocument | null> {
    return this.courseModel
      .findByIdAndUpdate(courseId, { $pull: { coInstructorIds: coInstructorId } }, { new: true })
      .exec();
  }

  async publish(id: string): Promise<CourseDocument | null> {
    return this.courseModel.findByIdAndUpdate(id, { isPublished: true }, { new: true }).exec();
  }

  async unpublish(id: string): Promise<CourseDocument | null> {
    return this.courseModel.findByIdAndUpdate(id, { isPublished: false }, { new: true }).exec();
  }

  async findByCrossListingGroup(groupId: string): Promise<CourseDocument[]> {
    return this.courseModel.find({ crossListingGroupId: groupId }).exec();
  }

  async findByIds(ids: string[]): Promise<CourseDocument[]> {
    return this.courseModel.find({ _id: { $in: ids } }).exec();
  }
}
