'use client';

import { useState } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import {
  TrendingUp,
  TrendingDown,
  Users,
  BookOpen,
  Award,
  Clock,
  BarChart3,
  PieChart,
  Calendar,
  Download
} from 'lucide-react';
import { mockCourses, mockAssignments } from '@/lib/mock-data';

export default function AnalyticsPage() {
  const [timeRange, setTimeRange] = useState<'week' | 'month' | 'semester'>('month');

  // Mock analytics data
  const totalStudents = mockCourses.reduce((sum, c) => sum + c.enrollmentCount, 0);
  const avgCourseProgress = mockCourses.reduce((sum, c) => sum + c.progress, 0) / mockCourses.length;
  const completionRate = 78.5;
  const avgGrade = 82.3;

  const courseEngagement = mockCourses.map(course => ({
    name: course.code,
    engagement: Math.floor(Math.random() * 40) + 60,
    students: course.enrollmentCount,
  }));

  const weeklyActivity = [
    { day: 'Mon', submissions: 45, logins: 120 },
    { day: 'Tue', submissions: 52, logins: 135 },
    { day: 'Wed', submissions: 38, logins: 110 },
    { day: 'Thu', submissions: 61, logins: 145 },
    { day: 'Fri', submissions: 43, logins: 98 },
    { day: 'Sat', submissions: 28, logins: 65 },
    { day: 'Sun', submissions: 31, logins: 72 },
  ];

  const maxSubmissions = Math.max(...weeklyActivity.map(d => d.submissions));
  const maxLogins = Math.max(...weeklyActivity.map(d => d.logins));

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-emerald-50 to-teal-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
              Analytics Dashboard
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mt-1">
              Comprehensive insights into student performance and engagement
            </p>
          </div>
          <div className="flex gap-2">
            <Button
              variant={timeRange === 'week' ? 'default' : 'outline'}
              onClick={() => setTimeRange('week')}
              className={timeRange === 'week' ? 'bg-emerald-600 hover:bg-emerald-700' : ''}
            >
              Week
            </Button>
            <Button
              variant={timeRange === 'month' ? 'default' : 'outline'}
              onClick={() => setTimeRange('month')}
              className={timeRange === 'month' ? 'bg-emerald-600 hover:bg-emerald-700' : ''}
            >
              Month
            </Button>
            <Button
              variant={timeRange === 'semester' ? 'default' : 'outline'}
              onClick={() => setTimeRange('semester')}
              className={timeRange === 'semester' ? 'bg-emerald-600 hover:bg-emerald-700' : ''}
            >
              Semester
            </Button>
            <Button className="bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-700 hover:to-teal-700 text-white">
              <Download className="h-4 w-4 mr-2" />
              Export
            </Button>
          </div>
        </div>

        {/* Key Metrics */}
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Total Students</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {totalStudents}
                </p>
                <div className="flex items-center gap-1 mt-2 text-emerald-600 dark:text-emerald-400">
                  <TrendingUp className="h-4 w-4" />
                  <span className="text-sm font-medium">+12.5%</span>
                </div>
              </div>
              <Users className="h-10 w-10 text-emerald-500" />
            </div>
          </Card>

          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Avg Progress</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {avgCourseProgress.toFixed(1)}%
                </p>
                <div className="flex items-center gap-1 mt-2 text-emerald-600 dark:text-emerald-400">
                  <TrendingUp className="h-4 w-4" />
                  <span className="text-sm font-medium">+8.2%</span>
                </div>
              </div>
              <BookOpen className="h-10 w-10 text-indigo-500" />
            </div>
          </Card>

          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Completion Rate</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {completionRate}%
                </p>
                <div className="flex items-center gap-1 mt-2 text-red-600 dark:text-red-400">
                  <TrendingDown className="h-4 w-4" />
                  <span className="text-sm font-medium">-2.1%</span>
                </div>
              </div>
              <Award className="h-10 w-10 text-amber-500" />
            </div>
          </Card>

          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Average Grade</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {avgGrade}%
                </p>
                <div className="flex items-center gap-1 mt-2 text-emerald-600 dark:text-emerald-400">
                  <TrendingUp className="h-4 w-4" />
                  <span className="text-sm font-medium">+5.3%</span>
                </div>
              </div>
              <BarChart3 className="h-10 w-10 text-purple-500" />
            </div>
          </Card>
        </div>

        {/* Charts Row */}
        <div className="grid gap-6 lg:grid-cols-2">
          {/* Weekly Activity Chart */}
          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                  Weekly Activity
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Submissions and login trends
                </p>
              </div>
              <Calendar className="h-5 w-5 text-gray-400" />
            </div>
            <div className="space-y-4">
              {weeklyActivity.map((day) => (
                <div key={day.day} className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="font-medium text-gray-700 dark:text-gray-300 w-12">
                      {day.day}
                    </span>
                    <div className="flex-1 mx-4 flex gap-2">
                      <div className="flex-1 bg-gray-100 dark:bg-gray-700 rounded-full h-6 relative overflow-hidden">
                        <div
                          className="absolute inset-y-0 left-0 bg-gradient-to-r from-emerald-500 to-teal-500 rounded-full flex items-center justify-end pr-2"
                          style={{ width: `${(day.submissions / maxSubmissions) * 100}%` }}
                        >
                          <span className="text-xs font-medium text-white">
                            {day.submissions}
                          </span>
                        </div>
                      </div>
                      <div className="flex-1 bg-gray-100 dark:bg-gray-700 rounded-full h-6 relative overflow-hidden">
                        <div
                          className="absolute inset-y-0 left-0 bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full flex items-center justify-end pr-2"
                          style={{ width: `${(day.logins / maxLogins) * 100}%` }}
                        >
                          <span className="text-xs font-medium text-white">
                            {day.logins}
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
            <div className="flex items-center justify-center gap-6 mt-6 pt-6 border-t border-gray-200 dark:border-gray-700">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-gradient-to-r from-emerald-500 to-teal-500"></div>
                <span className="text-sm text-gray-600 dark:text-gray-400">Submissions</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-gradient-to-r from-indigo-500 to-purple-500"></div>
                <span className="text-sm text-gray-600 dark:text-gray-400">Logins</span>
              </div>
            </div>
          </Card>

          {/* Course Engagement */}
          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                  Course Engagement
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Student participation by course
                </p>
              </div>
              <PieChart className="h-5 w-5 text-gray-400" />
            </div>
            <div className="space-y-4">
              {courseEngagement.slice(0, 6).map((course, index) => (
                <div key={course.name} className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="font-medium text-gray-700 dark:text-gray-300">
                      {course.name}
                    </span>
                    <span className="text-gray-600 dark:text-gray-400">
                      {course.engagement}%
                    </span>
                  </div>
                  <div className="relative h-2 bg-gray-100 dark:bg-gray-700 rounded-full overflow-hidden">
                    <div
                      className={`absolute inset-y-0 left-0 rounded-full ${
                        index % 4 === 0
                          ? 'bg-gradient-to-r from-emerald-500 to-teal-500'
                          : index % 4 === 1
                          ? 'bg-gradient-to-r from-indigo-500 to-purple-500'
                          : index % 4 === 2
                          ? 'bg-gradient-to-r from-amber-500 to-orange-500'
                          : 'bg-gradient-to-r from-pink-500 to-rose-500'
                      }`}
                      style={{ width: `${course.engagement}%` }}
                    />
                  </div>
                  <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                    <Users className="h-3 w-3" />
                    <span>{course.students} students</span>
                  </div>
                </div>
              ))}
            </div>
          </Card>
        </div>

        {/* Performance Distribution */}
        <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Grade Distribution
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Student performance across all courses
              </p>
            </div>
            <BarChart3 className="h-5 w-5 text-gray-400" />
          </div>
          <div className="grid grid-cols-5 gap-4">
            {[
              { grade: 'A', count: 45, percentage: 28 },
              { grade: 'B', count: 62, percentage: 38 },
              { grade: 'C', count: 38, percentage: 24 },
              { grade: 'D', count: 12, percentage: 7 },
              { grade: 'F', count: 5, percentage: 3 },
            ].map((item) => (
              <div key={item.grade} className="flex flex-col items-center">
                <div className="w-full h-48 bg-gray-100 dark:bg-gray-700 rounded-lg relative overflow-hidden flex items-end">
                  <div
                    className={`w-full rounded-lg transition-all ${
                      item.grade === 'A'
                        ? 'bg-gradient-to-t from-emerald-500 to-teal-500'
                        : item.grade === 'B'
                        ? 'bg-gradient-to-t from-indigo-500 to-purple-500'
                        : item.grade === 'C'
                        ? 'bg-gradient-to-t from-amber-500 to-yellow-500'
                        : item.grade === 'D'
                        ? 'bg-gradient-to-t from-orange-500 to-red-500'
                        : 'bg-gradient-to-t from-red-600 to-red-700'
                    }`}
                    style={{ height: `${item.percentage * 3}%` }}
                  />
                </div>
                <div className="mt-3 text-center">
                  <div className="text-2xl font-bold text-gray-900 dark:text-white">
                    {item.grade}
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    {item.count} students
                  </div>
                  <div className="text-xs text-gray-500 dark:text-gray-500">
                    {item.percentage}%
                  </div>
                </div>
              </div>
            ))}
          </div>
        </Card>

        {/* Top Performing Students */}
        <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Top Performing Students
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Based on overall grade average
              </p>
            </div>
            <Award className="h-5 w-5 text-amber-500" />
          </div>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="border-b border-gray-200 dark:border-gray-700">
                <tr>
                  <th className="pb-3 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Rank
                  </th>
                  <th className="pb-3 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Student
                  </th>
                  <th className="pb-3 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    GPA
                  </th>
                  <th className="pb-3 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Courses
                  </th>
                  <th className="pb-3 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Assignments
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {Array.from({ length: 10 }, (_, i) => ({
                  rank: i + 1,
                  name: `Student ${i + 1}`,
                  id: `S${1000 + i}`,
                  gpa: (4.0 - i * 0.15).toFixed(2),
                  courses: Math.floor(Math.random() * 3) + 4,
                  assignments: Math.floor(Math.random() * 10) + 15,
                })).map((student) => (
                  <tr key={student.id} className="hover:bg-gray-50 dark:hover:bg-gray-900">
                    <td className="py-4">
                      <div className="flex items-center justify-center w-8 h-8 rounded-full bg-gradient-to-r from-amber-500 to-yellow-500 text-white font-bold text-sm">
                        {student.rank}
                      </div>
                    </td>
                    <td className="py-4">
                      <div className="font-medium text-gray-900 dark:text-white">
                        {student.name}
                      </div>
                      <div className="text-sm text-gray-600 dark:text-gray-400">
                        {student.id}
                      </div>
                    </td>
                    <td className="py-4">
                      <span className="text-lg font-semibold text-gray-900 dark:text-white">
                        {student.gpa}
                      </span>
                    </td>
                    <td className="py-4 text-gray-900 dark:text-white">
                      {student.courses}
                    </td>
                    <td className="py-4 text-gray-900 dark:text-white">
                      {student.assignments}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      </div>
    </div>
  );
}
