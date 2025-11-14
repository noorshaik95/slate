import { Injectable } from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { CourseTemplate, CourseTemplateDocument } from '../schemas/course-template.schema';

@Injectable()
export class CourseTemplateRepository {
  constructor(
    @InjectModel(CourseTemplate.name) private templateModel: Model<CourseTemplateDocument>,
  ) {}

  async create(template: Partial<CourseTemplate>): Promise<CourseTemplateDocument> {
    const createdTemplate = new this.templateModel(template);
    return createdTemplate.save();
  }

  async findById(id: string): Promise<CourseTemplateDocument | null> {
    return this.templateModel.findById(id).exec();
  }

  async findAll(filters: {
    page?: number;
    pageSize?: number;
  }): Promise<{ templates: CourseTemplateDocument[]; total: number }> {
    const page = filters.page || 1;
    const pageSize = filters.pageSize || 20;
    const skip = (page - 1) * pageSize;

    const [templates, total] = await Promise.all([
      this.templateModel.find().skip(skip).limit(pageSize).exec(),
      this.templateModel.countDocuments().exec(),
    ]);

    return { templates, total };
  }

  async update(
    id: string,
    updates: Partial<CourseTemplate>,
  ): Promise<CourseTemplateDocument | null> {
    return this.templateModel.findByIdAndUpdate(id, updates, { new: true }).exec();
  }

  async delete(id: string): Promise<boolean> {
    const result = await this.templateModel.findByIdAndDelete(id).exec();
    return result !== null;
  }
}
