import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document } from 'mongoose';
import { CourseMetadata, CourseMetadataSchema } from './course.schema';

export type CourseTemplateDocument = CourseTemplate & Document;

@Schema({ timestamps: true })
export class CourseTemplate {
  @Prop({ required: true })
  name: string;

  @Prop({ required: true })
  description: string;

  @Prop()
  syllabusTemplate?: string;

  @Prop({ required: true })
  createdBy: string;

  @Prop({ type: CourseMetadataSchema })
  defaultMetadata?: CourseMetadata;
}

export const CourseTemplateSchema = SchemaFactory.createForClass(CourseTemplate);

// Indexes
CourseTemplateSchema.index({ name: 1 });
CourseTemplateSchema.index({ createdBy: 1 });
