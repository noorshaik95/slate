import { UpcomingAssignmentsWidget } from '@/components/dashboard/upcoming-assignments-widget';
import { RecentGradesWidget } from '@/components/dashboard/recent-grades-widget';
import { AnnouncementsWidget } from '@/components/dashboard/announcements-widget';
import { CalendarWidget } from '@/components/dashboard/calendar-widget';
import { QuickActionsWidget } from '@/components/dashboard/quick-actions-widget';
import { CourseProgressWidget } from '@/components/dashboard/course-progress-widget';
import { AcademicPerformanceWidget } from '@/components/dashboard/academic-performance-widget';

export const metadata = {
  title: 'Dashboard | Student Portal',
  description: 'View your courses, assignments, grades, and more',
};

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      {/* Welcome Section */}
      <div className="mb-8">
        <h1 className="text-4xl font-bold tracking-tight text-gray-900 dark:text-white mb-2">
          Welcome back, John!
        </h1>
        <p className="text-lg text-gray-600 dark:text-gray-400">
          Here&apos;s what&apos;s happening with your courses today.
        </p>
      </div>

      {/* Quick Actions */}
      <QuickActionsWidget />

      {/* Main Dashboard Grid */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {/* Course Progress */}
        <div className="lg:col-span-2">
          <CourseProgressWidget />
        </div>

        {/* Calendar Widget */}
        <CalendarWidget />

        {/* Academic Performance Widget */}
        <div className="md:col-span-2 lg:col-span-1">
          <AcademicPerformanceWidget />
        </div>

        {/* Upcoming Assignments */}
        <UpcomingAssignmentsWidget />

        {/* Recent Grades */}
        <RecentGradesWidget />

        {/* Announcements */}
        <AnnouncementsWidget />
      </div>
    </div>
  );
}
