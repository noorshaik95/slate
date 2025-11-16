import * as React from 'react';
import { cn } from '@/lib/utils';
import { GradientType } from './gradient-card';
import { AnimatedProgress } from './animated-progress';
import { FileText, Calendar } from 'lucide-react';

interface GradeCardProps {
  course: {
    id: string;
    code: string;
    name: string;
    instructor: string;
    gradient: GradientType;
    grade: number;
    progress: number;
  };
  assignments: Array<{
    id: string;
    name: string;
    type: string;
    grade: number;
    maxPoints: number;
    date: Date;
    feedback?: string;
  }>;
  className?: string;
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

const gradientTextClasses: Record<GradientType, string> = {
  'blue-cyan': 'text-cyan-600 dark:text-cyan-400',
  'purple-pink': 'text-pink-600 dark:text-pink-400',
  'emerald-teal': 'text-teal-600 dark:text-teal-400',
  'orange-red': 'text-red-600 dark:text-red-400',
  'amber-yellow': 'text-yellow-600 dark:text-yellow-400',
  'violet-indigo': 'text-indigo-600 dark:text-indigo-400',
  'indigo-purple': 'text-purple-600 dark:text-purple-400',
};

const gradientBgClasses: Record<GradientType, string> = {
  'blue-cyan': 'bg-cyan-50 dark:bg-cyan-900/20 border-cyan-200 dark:border-cyan-800',
  'purple-pink': 'bg-pink-50 dark:bg-pink-900/20 border-pink-200 dark:border-pink-800',
  'emerald-teal': 'bg-teal-50 dark:bg-teal-900/20 border-teal-200 dark:border-teal-800',
  'orange-red': 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800',
  'amber-yellow': 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800',
  'violet-indigo': 'bg-indigo-50 dark:bg-indigo-900/20 border-indigo-200 dark:border-indigo-800',
  'indigo-purple': 'bg-purple-50 dark:bg-purple-900/20 border-purple-200 dark:border-purple-800',
};

const formatDate = (date: Date): string => {
  const options: Intl.DateTimeFormatOptions = {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  };
  return date.toLocaleDateString('en-US', options);
};

export const GradeCard = React.forwardRef<HTMLDivElement, GradeCardProps>(
  ({ course, assignments, className }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          'bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700',
          'shadow-sm p-6',
          className
        )}
      >
        {/* Course Header */}
        <div className="flex items-start justify-between mb-4">
          <div className="flex-1">
            <div
              className={cn(
                'inline-block px-3 py-1 rounded-lg text-white font-semibold text-sm mb-2',
                gradientBadgeClasses[course.gradient]
              )}
            >
              {course.code}
            </div>
            <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-1">
              {course.name}
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {course.instructor}
            </p>
          </div>
          <div className="text-right">
            <div className={cn('text-4xl font-bold', gradientTextClasses[course.gradient])}>
              {course.grade}%
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
              Current Grade
            </div>
          </div>
        </div>

        {/* Course Progress */}
        <div className="mb-6">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Course Progress
            </span>
            <span className="text-sm font-semibold text-gray-900 dark:text-white">
              {course.progress}%
            </span>
          </div>
          <AnimatedProgress
            value={course.progress}
            gradient={course.gradient}
            size="md"
            animated={true}
          />
        </div>

        {/* Graded Assignments */}
        <div>
          <h4 className="text-sm font-semibold text-gray-900 dark:text-white mb-3">
            Graded Assignments
          </h4>
          <div className="space-y-3">
            {assignments.map((assignment) => (
              <div
                key={assignment.id}
                className={cn(
                  'rounded-lg border p-4',
                  gradientBgClasses[course.gradient]
                )}
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="flex items-start gap-2 flex-1">
                    <FileText className="w-4 h-4 mt-0.5 text-gray-600 dark:text-gray-400" />
                    <div>
                      <h5 className="font-semibold text-gray-900 dark:text-white text-sm">
                        {assignment.name}
                      </h5>
                      <div className="flex items-center gap-2 mt-1 text-xs text-gray-600 dark:text-gray-400">
                        <Calendar className="w-3 h-3" />
                        <span>{formatDate(assignment.date)}</span>
                        <span>•</span>
                        <span className="capitalize">{assignment.type}</span>
                      </div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className={cn('text-lg font-bold', gradientTextClasses[course.gradient])}>
                      {assignment.grade}/{assignment.maxPoints}
                    </div>
                    <div className="text-xs text-gray-600 dark:text-gray-400">
                      {Math.round((assignment.grade / assignment.maxPoints) * 100)}%
                    </div>
                  </div>
                </div>
                {assignment.feedback && (
                  <div className="mt-2 pt-2 border-t border-gray-200 dark:border-gray-700">
                    <p className="text-sm text-emerald-700 dark:text-emerald-400 italic">
                      "{assignment.feedback}"
                    </p>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }
);

GradeCard.displayName = 'GradeCard';
