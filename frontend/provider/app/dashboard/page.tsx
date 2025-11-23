import { CourseOverviewWidget } from '@/components/dashboard/course-overview-widget';
import { GradingQueueWidget } from '@/components/dashboard/grading-queue-widget';
import { UpcomingClassesWidget } from '@/components/dashboard/upcoming-classes-widget';
import { RecentActivityWidget } from '@/components/dashboard/recent-activity-widget';
import { StudentPerformanceWidget } from '@/components/dashboard/student-performance-widget';
import { mockStats } from '@/lib/mock-data';

export const metadata = {
  title: 'Dashboard | Instructor Portal',
  description: 'View your courses, grading queue, student performance, and more',
};

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      {/* Dashboard Grid */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {/* Quick Stats Cards */}
        <div className="bg-gradient-to-br from-blue-500 to-cyan-500 rounded-2xl p-6 text-white shadow-lg hover-lift">
          <h3 className="text-sm font-medium mb-2 opacity-90">Active Courses</h3>
          <p className="text-4xl font-bold">{mockStats.totalCourses}</p>
          <p className="text-sm mt-2 opacity-75">{mockStats.totalStudents} total students</p>
        </div>

        <div className="bg-gradient-to-br from-purple-500 to-pink-500 rounded-2xl p-6 text-white shadow-lg hover-lift">
          <h3 className="text-sm font-medium mb-2 opacity-90">To Grade</h3>
          <p className="text-4xl font-bold">{mockStats.toGrade}</p>
          <p className="text-sm mt-2 opacity-75">assignments pending</p>
        </div>

        <div className="bg-gradient-to-br from-emerald-500 to-teal-500 rounded-2xl p-6 text-white shadow-lg hover-lift">
          <h3 className="text-sm font-medium mb-2 opacity-90">Average Grade</h3>
          <p className="text-4xl font-bold">{mockStats.averageGrade}%</p>
          <p className="text-sm mt-2 opacity-75">across all courses</p>
        </div>
      </div>

      {/* Main Dashboard Grid */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* Left Column - Course Overview and Grading Queue */}
        <div className="lg:col-span-2 space-y-6">
          <CourseOverviewWidget />
          <RecentActivityWidget />
        </div>

        {/* Right Column - Upcoming Classes and Grading Queue */}
        <div className="space-y-6">
          <UpcomingClassesWidget />
          <GradingQueueWidget />
        </div>
      </div>

      {/* Analytics Row */}
      <div className="grid gap-6">
        <StudentPerformanceWidget />
      </div>
    </div>
  );
}
