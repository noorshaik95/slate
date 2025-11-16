import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document } from 'mongoose';

export type CrossListingDocument = CrossListing & Document;

@Schema({ timestamps: true })
export class CrossListing {
  @Prop({ required: true })
  groupId: string;

  @Prop({ type: [String], required: true })
  courseIds: string[];

  @Prop({ required: true })
  createdBy: string;

  createdAt?: Date;
  updatedAt?: Date;
}

export const CrossListingSchema = SchemaFactory.createForClass(CrossListing);

// Indexes
CrossListingSchema.index({ groupId: 1 }, { unique: true });
CrossListingSchema.index({ courseIds: 1 });
