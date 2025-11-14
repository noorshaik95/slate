import { Injectable } from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { CrossListing, CrossListingDocument } from '../schemas/cross-listing.schema';

@Injectable()
export class CrossListingRepository {
  constructor(
    @InjectModel(CrossListing.name) private crossListingModel: Model<CrossListingDocument>,
  ) {}

  async create(crossListing: Partial<CrossListing>): Promise<CrossListingDocument> {
    const createdCrossListing = new this.crossListingModel(crossListing);
    return createdCrossListing.save();
  }

  async findByGroupId(groupId: string): Promise<CrossListingDocument | null> {
    return this.crossListingModel.findOne({ groupId }).exec();
  }

  async findByCourseId(courseId: string): Promise<CrossListingDocument | null> {
    return this.crossListingModel.findOne({ courseIds: courseId }).exec();
  }

  async delete(groupId: string): Promise<boolean> {
    const result = await this.crossListingModel.findOneAndDelete({ groupId }).exec();
    return result !== null;
  }

  async addCourse(groupId: string, courseId: string): Promise<CrossListingDocument | null> {
    return this.crossListingModel
      .findOneAndUpdate(groupId, { $addToSet: { courseIds: courseId } }, { new: true })
      .exec();
  }

  async removeCourse(groupId: string, courseId: string): Promise<CrossListingDocument | null> {
    return this.crossListingModel
      .findOneAndUpdate(groupId, { $pull: { courseIds: courseId } }, { new: true })
      .exec();
  }
}
