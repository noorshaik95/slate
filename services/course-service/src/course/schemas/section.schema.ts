import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document, Types } from 'mongoose';

export type SectionDocument = Section & Document;

@Schema({ _id: false })
export class Schedule {
  @Prop({ type: [String], required: true })
  daysOfWeek: string[]; // e.g., ['MON', 'WED', 'FRI']

  @Prop({ required: true })
  startTime: string; // e.g., '09:00'

  @Prop({ required: true })
  endTime: string; // e.g., '10:30'
}

export const ScheduleSchema = SchemaFactory.createForClass(Schedule);

@Schema({ timestamps: true })
export class Section {
  @Prop({ type: Types.ObjectId, ref: 'Course', required: true })
  courseId: string;

  @Prop({ required: true })
  sectionNumber: string;

  @Prop({ required: true })
  instructorId: string;

  @Prop({ type: ScheduleSchema, required: true })
  schedule: Schedule;

  @Prop()
  location?: string;

  @Prop({ default: 0 })
  maxStudents: number;

  @Prop({ default: 0 })
  enrolledCount: number;
}

export const SectionSchema = SchemaFactory.createForClass(Section);

// Indexes
SectionSchema.index({ courseId: 1, sectionNumber: 1 }, { unique: true });
SectionSchema.index({ instructorId: 1 });
