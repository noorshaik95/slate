'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { CourseCard } from '@/components/common/course-card';
import { BookOpen, Search } from 'lucide-react';
import { mockCourses } from '@/lib/mock-data';
import { GradientType } from '@/components/common/gradient-card';

// Map course colors to gradient types
const colorToGradient: Record<string, GradientType> = {
  blue: 'blue-cyan',
  purple: 'purple-pink',
  green: 'emerald-teal',
  red: 'orange-red',
  amber: 'amber-yellow',
  violet: 'violet-indigo',
  indigo: 'indigo-purple',
};

export default function CoursesPage() {
  const router = useRouter();
  const [searchQuery, setSearchQuery] = useState('');

  // Transform courses to include gradient
  const coursesWithGradient = mockCourses.map((course) => ({
    ...course,
    gradient: colorToGradient[course.color] || 'blue-cyan',
    studentCount: course.enrollmentCount,
  }));

  // Filter courses based on search query
  const filteredCourses = coursesWithGradient.filter(
    (course) =>
      course.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      course.code.toLowerCase().includes(searchQuery.toLowerCase()) ||
      course.instructor.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
            My Courses
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            You are enrolled in {mockCourses.length} courses this semester
          </p>
        </div>

        {/* Search Bar */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-gray-400" />
          <Input
            type="text"
            placeholder="Search courses by name, code, or instructor..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10 rounded-xl border-gray-300 dark:border-gray-700 focus:border-indigo-500 focus:ring-indigo-500"
          />
        </div>

        {/* Course Grid */}
        {filteredCourses.length > 0 ? (
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
            {filteredCourses.map((course) => (
              <CourseCard
                key={course.id}
                course={course}
                variant="detailed"
                showProgress={true}
                showMetadata={true}
                onContinue={() => router.push(`/courses/${course.id}`)}
              />
            ))}
          </div>
        ) : (
          <Card className="p-12 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl">
            <div className="flex flex-col items-center text-center space-y-4">
              <BookOpen className="h-16 w-16 text-gray-400 dark:text-gray-600" />
              <div className="space-y-2">
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white">
                  No Courses Found
                </h3>
                <p className="text-gray-600 dark:text-gray-400">
                  {searchQuery
                    ? 'Try adjusting your search terms'
                    : "You haven't enrolled in any courses. Contact your advisor to get started."}
                </p>
              </div>
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}
