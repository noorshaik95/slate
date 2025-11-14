import { Injectable } from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import {
  Enrollment,
  EnrollmentDocument,
  EnrollmentStatus,
} from '../schemas/enrollment.schema';

@Injectable()
export class EnrollmentRepository {
  constructor(
    @InjectModel(Enrollment.name) private enrollmentModel: Model<EnrollmentDocument>,
  ) {}

  async create(enrollment: Partial<Enrollment>): Promise<EnrollmentDocument> {
    const createdEnrollment = new this.enrollmentModel(enrollment);
    return createdEnrollment.save();
  }

  async findById(id: string): Promise<EnrollmentDocument | null> {
    return this.enrollmentModel.findById(id).exec();
  }

  async findByCourseAndStudent(
    courseId: string,
    studentId: string,
  ): Promise<EnrollmentDocument | null> {
    return this.enrollmentModel.findOne({ courseId, studentId }).exec();
  }

  async findByCourse(
    courseId: string,
    filters?: {
      sectionId?: string;
      status?: EnrollmentStatus;
    },
  ): Promise<EnrollmentDocument[]> {
    const query: any = { courseId };

    if (filters?.sectionId) {
      query.sectionId = filters.sectionId;
    }

    if (filters?.status) {
      query.status = filters.status;
    }

    return this.enrollmentModel.find(query).exec();
  }

  async findByStudent(
    studentId: string,
    filters?: {
      term?: string;
      status?: EnrollmentStatus;
    },
  ): Promise<EnrollmentDocument[]> {
    const query: any = { studentId };

    if (filters?.status) {
      query.status = filters.status;
    }

    // Note: For term filtering, we'll need to populate the course
    const enrollments = await this.enrollmentModel.find(query).populate('courseId').exec();

    if (filters?.term) {
      return enrollments.filter((e: any) => e.courseId?.term === filters.term);
    }

    return enrollments;
  }

  async delete(id: string): Promise<boolean> {
    const result = await this.enrollmentModel.findByIdAndDelete(id).exec();
    return result !== null;
  }

  async updateStatus(id: string, status: EnrollmentStatus): Promise<EnrollmentDocument | null> {
    return this.enrollmentModel.findByIdAndUpdate(id, { status }, { new: true }).exec();
  }

  async countByCourse(courseId: string, status?: EnrollmentStatus): Promise<number> {
    const query: any = { courseId };

    if (status) {
      query.status = status;
    }

    return this.enrollmentModel.countDocuments(query).exec();
  }

  async countBySection(sectionId: string, status?: EnrollmentStatus): Promise<number> {
    const query: any = { sectionId };

    if (status) {
      query.status = status;
    }

    return this.enrollmentModel.countDocuments(query).exec();
  }
}
