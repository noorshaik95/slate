import { Injectable } from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { Section, SectionDocument } from '../schemas/section.schema';

@Injectable()
export class SectionRepository {
  constructor(@InjectModel(Section.name) private sectionModel: Model<SectionDocument>) {}

  async create(section: Partial<Section>): Promise<SectionDocument> {
    const createdSection = new this.sectionModel(section);
    return createdSection.save();
  }

  async findById(id: string): Promise<SectionDocument | null> {
    return this.sectionModel.findById(id).exec();
  }

  async findByCourse(courseId: string): Promise<SectionDocument[]> {
    return this.sectionModel.find({ courseId }).exec();
  }

  async findByCourseAndNumber(
    courseId: string,
    sectionNumber: string,
  ): Promise<SectionDocument | null> {
    return this.sectionModel.findOne({ courseId, sectionNumber }).exec();
  }

  async update(id: string, updates: Partial<Section>): Promise<SectionDocument | null> {
    return this.sectionModel.findByIdAndUpdate(id, updates, { new: true }).exec();
  }

  async delete(id: string): Promise<boolean> {
    const result = await this.sectionModel.findByIdAndDelete(id).exec();
    return result !== null;
  }

  async incrementEnrolledCount(id: string): Promise<SectionDocument | null> {
    return this.sectionModel
      .findByIdAndUpdate(id, { $inc: { enrolledCount: 1 } }, { new: true })
      .exec();
  }

  async decrementEnrolledCount(id: string): Promise<SectionDocument | null> {
    return this.sectionModel
      .findByIdAndUpdate(id, { $inc: { enrolledCount: -1 } }, { new: true })
      .exec();
  }
}
