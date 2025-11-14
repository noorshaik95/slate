import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document, Types } from 'mongoose';

export type CourseDocument = Course & Document;

@Schema({ collection: 'course_metadata' })
export class CourseMetadata {
  @Prop()
  maxStudents?: number;

  @Prop()
  department?: string;

  @Prop()
  courseCode?: string;

  @Prop()
  credits?: number;

  @Prop([String])
  tags?: string[];
}

export const CourseMetadataSchema = SchemaFactory.createForClass(CourseMetadata);

@Schema({ timestamps: true })
export class Course {
  @Prop({ required: true })
  title: string;

  @Prop({ required: true })
  description: string;

  @Prop({ required: true })
  term: string;

  @Prop({ type: String })
  syllabus?: string;

  @Prop({ required: true })
  instructorId: string;

  @Prop({ type: [String], default: [] })
  coInstructorIds: string[];

  @Prop({ default: false })
  isPublished: boolean;

  @Prop({ type: [String], default: [] })
  prerequisiteCourseIds: string[];

  @Prop({ type: Types.ObjectId, ref: 'CourseTemplate' })
  templateId?: string;

  @Prop()
  crossListingGroupId?: string;

  @Prop({ type: CourseMetadataSchema })
  metadata?: CourseMetadata;
}

export const CourseSchema = SchemaFactory.createForClass(Course);

// Indexes
CourseSchema.index({ instructorId: 1, term: 1 });
CourseSchema.index({ isPublished: 1 });
CourseSchema.index({ term: 1 });
CourseSchema.index({ 'metadata.department': 1, 'metadata.courseCode': 1 });
CourseSchema.index({ crossListingGroupId: 1 });
