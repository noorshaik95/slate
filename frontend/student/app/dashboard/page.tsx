import { UpcomingAssignmentsWidget } from '@/components/dashboard/upcoming-assignments-widget';
import { RecentGradesWidget } from '@/components/dashboard/recent-grades-widget';
import { AnnouncementsWidget } from '@/components/dashboard/announcements-widget';
import { CalendarWidget } from '@/components/dashboard/calendar-widget';
import { QuickActionsWidget } from '@/components/dashboard/quick-actions-widget';
import { CourseProgressWidget } from '@/components/dashboard/course-progress-widget';

export const metadata = {
  title: 'Dashboard | Student Portal',
  description: 'View your courses, assignments, grades, and more',
};

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      {/* Welcome Section */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Welcome back, John!</h1>
        <p className="text-muted-foreground">
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
