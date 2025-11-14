import { Injectable, Logger, NotFoundException, BadRequestException } from '@nestjs/common';
import { CourseRepository } from './repositories/course.repository';
import { CourseTemplateRepository } from './repositories/course-template.repository';
import { SectionRepository } from './repositories/section.repository';
import { CrossListingRepository } from './repositories/cross-listing.repository';
import { MetricsService } from '../observability/metrics.service';
import { Course } from './schemas/course.schema';
import { v4 as uuidv4 } from 'uuid';

@Injectable()
export class CourseService {
  private readonly logger = new Logger(CourseService.name);

  constructor(
    private readonly courseRepository: CourseRepository,
    private readonly templateRepository: CourseTemplateRepository,
    private readonly sectionRepository: SectionRepository,
    private readonly crossListingRepository: CrossListingRepository,
    private readonly metricsService: MetricsService,
  ) {}

  async createCourse(courseData: Partial<Course>) {
    try {
      this.logger.log(`Creating course: ${courseData.title}`);
      const course = await this.courseRepository.create(courseData);
      this.metricsService.incrementCoursesCreated(true);
      this.logger.log(`Course created successfully: ${course._id}`);
      return course;
    } catch (error) {
      this.metricsService.incrementCoursesCreated(false);
      this.logger.error(`Failed to create course: ${error.message}`, error.stack);
      throw error;
    }
  }

  async getCourse(courseId: string) {
    const course = await this.courseRepository.findById(courseId);
    if (!course) {
      throw new NotFoundException(`Course not found: ${courseId}`);
    }
    return course;
  }

  async listCourses(filters: {
    instructorId?: string;
    term?: string;
    isPublished?: boolean;
    page?: number;
    pageSize?: number;
  }) {
    return this.courseRepository.findAll(filters);
  }

  async updateCourse(courseId: string, updates: Partial<Course>) {
    const course = await this.courseRepository.update(courseId, updates);
    if (!course) {
      throw new NotFoundException(`Course not found: ${courseId}`);
    }
    this.logger.log(`Course updated: ${courseId}`);
    return course;
  }

  async deleteCourse(courseId: string) {
    const deleted = await this.courseRepository.delete(courseId);
    if (!deleted) {
      throw new NotFoundException(`Course not found: ${courseId}`);
    }
    this.logger.log(`Course deleted: ${courseId}`);
    return true;
  }

  async publishCourse(courseId: string) {
    const course = await this.courseRepository.publish(courseId);
    if (!course) {
      throw new NotFoundException(`Course not found: ${courseId}`);
    }
    this.metricsService.incrementCoursesPublished();
    this.logger.log(`Course published: ${courseId}`);
    return course;
  }

  async unpublishCourse(courseId: string) {
    const course = await this.courseRepository.unpublish(courseId);
    if (!course) {
      throw new NotFoundException(`Course not found: ${courseId}`);
    }
    this.metricsService.incrementCoursesUnpublished();
    this.logger.log(`Course unpublished: ${courseId}`);
    return course;
  }

  // Template operations
  async createTemplate(templateData: any) {
    const template = await this.templateRepository.create(templateData);
    this.metricsService.incrementTemplatesCreated();
    this.logger.log(`Template created: ${template._id}`);
    return template;
  }

  async getTemplate(templateId: string) {
    const template = await this.templateRepository.findById(templateId);
    if (!template) {
      throw new NotFoundException(`Template not found: ${templateId}`);
    }
    return template;
  }

  async listTemplates(filters: { page?: number; pageSize?: number }) {
    return this.templateRepository.findAll(filters);
  }

  async createCourseFromTemplate(
    templateId: string,
    courseData: { title: string; term: string; instructorId: string; description?: string },
  ) {
    const template = await this.getTemplate(templateId);

    const newCourse = await this.createCourse({
      title: courseData.title,
      description: courseData.description || template.description,
      term: courseData.term,
      syllabus: template.syllabusTemplate,
      instructorId: courseData.instructorId,
      templateId: template._id.toString(),
      metadata: template.defaultMetadata,
    });

    this.metricsService.incrementCoursesFromTemplates();
    this.logger.log(`Course created from template: ${newCourse._id}`);
    return newCourse;
  }

  // Prerequisites
  async addPrerequisite(courseId: string, prerequisiteId: string) {
    // Validate both courses exist
    await this.getCourse(courseId);
    await this.getCourse(prerequisiteId);

    // Check for circular dependencies
    await this.checkCircularPrerequisites(courseId, prerequisiteId);

    const course = await this.courseRepository.addPrerequisite(courseId, prerequisiteId);
    this.logger.log(`Prerequisite added to course ${courseId}: ${prerequisiteId}`);
    return course;
  }

  async removePrerequisite(courseId: string, prerequisiteId: string) {
    const course = await this.courseRepository.removePrerequisite(courseId, prerequisiteId);
    if (!course) {
      throw new NotFoundException(`Course not found: ${courseId}`);
    }
    this.logger.log(`Prerequisite removed from course ${courseId}: ${prerequisiteId}`);
    return course;
  }

  async checkPrerequisites(courseId: string, studentId: string) {
    const course = await this.getCourse(courseId);

    if (!course.prerequisiteCourseIds || course.prerequisiteCourseIds.length === 0) {
      return {
        hasPrerequisites: true,
        missingPrerequisiteIds: [],
        missingPrerequisites: [],
      };
    }

    // In a real implementation, you would check enrollment records
    // For now, we'll just return the prerequisites
    const prerequisites = await this.courseRepository.findByIds(course.prerequisiteCourseIds);

    // TODO: Check if student has completed these courses
    const missingPrerequisites = prerequisites;

    return {
      hasPrerequisites: missingPrerequisites.length === 0,
      missingPrerequisiteIds: missingPrerequisites.map((p) => p._id.toString()),
      missingPrerequisites,
    };
  }

  private async checkCircularPrerequisites(courseId: string, prerequisiteId: string) {
    const visited = new Set<string>();
    const stack = [prerequisiteId];

    while (stack.length > 0) {
      const currentId = stack.pop();
      if (currentId === courseId) {
        throw new BadRequestException('Circular prerequisite dependency detected');
      }

      if (visited.has(currentId)) {
        continue;
      }

      visited.add(currentId);
      const current = await this.courseRepository.findById(currentId);
      if (current?.prerequisiteCourseIds) {
        stack.push(...current.prerequisiteCourseIds);
      }
    }
  }

  // Co-teaching
  async addCoInstructor(courseId: string, coInstructorId: string) {
    const course = await this.courseRepository.addCoInstructor(courseId, coInstructorId);
    if (!course) {
      throw new NotFoundException(`Course not found: ${courseId}`);
    }
    this.logger.log(`Co-instructor added to course ${courseId}: ${coInstructorId}`);
    return course;
  }

  async removeCoInstructor(courseId: string, coInstructorId: string) {
    const course = await this.courseRepository.removeCoInstructor(courseId, coInstructorId);
    if (!course) {
      throw new NotFoundException(`Course not found: ${courseId}`);
    }
    this.logger.log(`Co-instructor removed from course ${courseId}: ${coInstructorId}`);
    return course;
  }

  // Sections
  async createSection(sectionData: any) {
    // Validate course exists
    await this.getCourse(sectionData.courseId);

    const section = await this.sectionRepository.create(sectionData);
    this.logger.log(`Section created: ${section._id}`);
    return section;
  }

  async getSection(sectionId: string) {
    const section = await this.sectionRepository.findById(sectionId);
    if (!section) {
      throw new NotFoundException(`Section not found: ${sectionId}`);
    }
    return section;
  }

  async updateSection(sectionId: string, updates: any) {
    const section = await this.sectionRepository.update(sectionId, updates);
    if (!section) {
      throw new NotFoundException(`Section not found: ${sectionId}`);
    }
    this.logger.log(`Section updated: ${sectionId}`);
    return section;
  }

  async deleteSection(sectionId: string) {
    const deleted = await this.sectionRepository.delete(sectionId);
    if (!deleted) {
      throw new NotFoundException(`Section not found: ${sectionId}`);
    }
    this.logger.log(`Section deleted: ${sectionId}`);
    return true;
  }

  async listCourseSections(courseId: string) {
    // Validate course exists
    await this.getCourse(courseId);
    return this.sectionRepository.findByCourse(courseId);
  }

  // Cross-listing
  async crossListCourses(courseIds: string[], createdBy: string) {
    // Validate all courses exist
    for (const courseId of courseIds) {
      await this.getCourse(courseId);
    }

    const groupId = uuidv4();

    const crossListing = await this.crossListingRepository.create({
      groupId,
      courseIds,
      createdBy,
    });

    // Update all courses with the cross-listing group ID
    for (const courseId of courseIds) {
      await this.courseRepository.update(courseId, { crossListingGroupId: groupId });
    }

    this.logger.log(`Cross-listing created: ${groupId}`);
    return crossListing;
  }

  async removeCrossListing(groupId: string) {
    const crossListing = await this.crossListingRepository.findByGroupId(groupId);
    if (!crossListing) {
      throw new NotFoundException(`Cross-listing not found: ${groupId}`);
    }

    // Remove cross-listing group ID from all courses
    for (const courseId of crossListing.courseIds) {
      await this.courseRepository.update(courseId, { crossListingGroupId: null });
    }

    await this.crossListingRepository.delete(groupId);
    this.logger.log(`Cross-listing removed: ${groupId}`);
    return true;
  }

  async getCrossListedCourses(courseId: string) {
    const course = await this.getCourse(courseId);

    if (!course.crossListingGroupId) {
      return { courses: [course], groupId: null };
    }

    const courses = await this.courseRepository.findByCrossListingGroup(
      course.crossListingGroupId,
    );
    return { courses, groupId: course.crossListingGroupId };
  }
}
