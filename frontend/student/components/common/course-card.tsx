import * as React from 'react';
import { cn } from '@/lib/utils';
import { GradientType } from './gradient-card';
import { AnimatedProgress } from './animated-progress';
import { Button } from '@/components/ui/button';
import { ArrowRight, Users, BookOpen, Calendar } from 'lucide-react';

interface CourseCardProps {
  course: {
    id: string;
    code: string;
    name: string;
    instructor: string;
    progress: number;
    gradient: GradientType;
    studentCount?: number;
    credits?: number;
    nextDeadline?: string;
  };
  variant?: 'compact' | 'detailed';
  showProgress?: boolean;
  showMetadata?: boolean;
  className?: string;
  onContinue?: () => void;
}

const gradientBadgeClasses: Record<GradientType, string> = {
  'blue-cyan': 'gradient-blue-cyan',
  'purple-pink': 'gradient-purple-pink',
  'emerald-teal': 'gradient-emerald-teal',
  'orange-red': 'gradient-orange-red',
  'amber-yellow': 'gradient-amber-yellow',
  'violet-indigo': 'gradient-violet-indigo',
  'indigo-purple': 'gradient-indigo-purple',
};

export const CourseCard = React.forwardRef<HTMLDivElement, CourseCardProps>(
  (
    {
      course,
      variant = 'detailed',
      showProgress = true,
      showMetadata = true,
      className,
      onContinue,
    },
    ref
  ) => {
    return (
      <div
        ref={ref}
        className={cn(
          'bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700',
          'shadow-sm hover-lift transition-all duration-300',
          className
        )}
      >
        <div className="p-6">
          {/* Course Code Badge */}
          <div
            className={cn(
              'inline-block px-4 py-1.5 rounded-lg text-white font-semibold text-sm mb-4',
              gradientBadgeClasses[course.gradient]
            )}
          >
            {course.code}
          </div>

          {/* Course Name */}
          <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
            {course.name}
          </h3>

          {/* Instructor */}
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            {course.instructor}
          </p>

          {/* Metadata */}
          {showMetadata && variant === 'detailed' && (
            <div className="flex items-center gap-4 mb-4 text-sm text-gray-600 dark:text-gray-400">
              {course.studentCount !== undefined && (
                <div className="flex items-center gap-1.5">
                  <Users className="w-4 h-4" />
                  <span>{course.studentCount} students</span>
                </div>
              )}
              {course.credits !== undefined && (
                <div className="flex items-center gap-1.5">
                  <BookOpen className="w-4 h-4" />
                  <span>{course.credits} credits</span>
                </div>
              )}
              {course.nextDeadline && (
                <div className="flex items-center gap-1.5">
                  <Calendar className="w-4 h-4" />
                  <span>{course.nextDeadline}</span>
                </div>
              )}
            </div>
          )}

          {/* Progress Section */}
          {showProgress && (
            <div className="mb-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  Progress
                </span>
                <span className="text-2xl font-bold text-gray-900 dark:text-white">
                  {course.progress}%
                </span>
              </div>
              <AnimatedProgress
                value={course.progress}
                gradient={course.gradient}
                size="lg"
                animated={true}
              />
            </div>
          )}

          {/* Continue Button */}
          <Button
            onClick={onContinue}
            className="w-full mt-2"
            variant="default"
          >
            Continue
            <ArrowRight className="w-4 h-4" />
          </Button>
        </div>
      </div>
    );
  }
);

CourseCard.displayName = 'CourseCard';
