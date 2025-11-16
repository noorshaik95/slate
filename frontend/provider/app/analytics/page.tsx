'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  TrendingUp,
  TrendingDown,
  Users,
  Award,
  Activity,
  AlertTriangle,
} from 'lucide-react';
import { mockStudentPerformance, mockCourses } from '@/lib/mock-data';

export default function AnalyticsPage() {
  const { overall, gradeDistribution, engagementTrend, atRiskStudents } = mockStudentPerformance;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Analytics
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Track student performance and course metrics
        </p>
      </div>

      {/* Overview Stats */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-600 dark:text-gray-400 flex items-center gap-2">
              <Award className="h-4 w-4" />
              Average Grade
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              {overall.averageGrade}%
            </div>
            <div className="flex items-center gap-1 mt-1">
              <TrendingUp className="h-3 w-3 text-green-600" />
              <p className="text-xs text-green-600">+2.5% from last month</p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-600 dark:text-gray-400 flex items-center gap-2">
              <Users className="h-4 w-4" />
              Active Students
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              {overall.activeStudents}
            </div>
            <p className="text-xs text-gray-500 mt-1">
              of {overall.totalStudents} students ({Math.round((overall.activeStudents / overall.totalStudents) * 100)}%)
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-600 dark:text-gray-400 flex items-center gap-2">
              <Activity className="h-4 w-4" />
              Engagement Rate
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              93%
            </div>
            <div className="flex items-center gap-1 mt-1">
              <TrendingUp className="h-3 w-3 text-green-600" />
              <p className="text-xs text-green-600">+5% this week</p>
            </div>
          </CardContent>
        </Card>

        <Card className="border-orange-200 dark:border-orange-800">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-orange-600 dark:text-orange-400 flex items-center gap-2">
              <AlertTriangle className="h-4 w-4" />
              At-Risk Students
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-orange-600 dark:text-orange-400">
              {overall.atRiskStudents}
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Need immediate attention
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="performance" className="w-full">
        <TabsList>
          <TabsTrigger value="performance">Performance</TabsTrigger>
          <TabsTrigger value="engagement">Engagement</TabsTrigger>
          <TabsTrigger value="by-course">By Course</TabsTrigger>
        </TabsList>

        <TabsContent value="performance" className="space-y-6 mt-6">
          <div className="grid gap-6 md:grid-cols-2">
            {/* Grade Distribution */}
            <Card>
              <CardHeader>
                <CardTitle>Grade Distribution</CardTitle>
                <CardDescription>Overall grade breakdown across all courses</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {gradeDistribution.map((grade) => (
                    <div key={grade.grade} className="space-y-2">
                      <div className="flex items-center justify-between text-sm">
                        <div className="flex items-center gap-2">
                          <Badge variant={
                            grade.grade === 'A' ? 'success' :
                            grade.grade === 'B' ? 'info' :
                            grade.grade === 'C' ? 'default' :
                            grade.grade === 'D' ? 'warning' :
                            'destructive'
                          }>
                            {grade.grade}
                          </Badge>
                          <span className="font-medium text-gray-700 dark:text-gray-300">
                            {grade.count} students
                          </span>
                        </div>
                        <span className="text-gray-600 dark:text-gray-400">
                          {grade.percentage}%
                        </span>
                      </div>
                      <Progress value={grade.percentage} className="h-2" />
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* At-Risk Students */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <AlertTriangle className="h-5 w-5 text-orange-600" />
                  At-Risk Students
                </CardTitle>
                <CardDescription>Students who need immediate attention</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {atRiskStudents.map((student) => (
                    <div
                      key={student.id}
                      className="border border-orange-200 dark:border-orange-800 rounded-lg p-3 bg-orange-50 dark:bg-orange-950/20"
                    >
                      <div className="flex items-center gap-3">
                        <img
                          src={student.avatar}
                          alt={student.name}
                          className="w-10 h-10 rounded-full"
                        />
                        <div className="flex-1">
                          <div className="flex items-center justify-between">
                            <h5 className="text-sm font-semibold text-gray-900 dark:text-white">
                              {student.name}
                            </h5>
                            <Badge variant="outline" className="text-xs">
                              {student.currentGrade}%
                            </Badge>
                          </div>
                          <div className="flex items-center gap-3 mt-1 text-xs text-gray-600 dark:text-gray-400">
                            <Badge variant="outline" className="text-xs">
                              {student.courseName}
                            </Badge>
                            <span>{student.missedAssignments} missed</span>
                            <Badge variant={student.riskLevel === 'high' ? 'destructive' : 'warning'} className="text-xs">
                              {student.riskLevel}
                            </Badge>
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Performance Trends */}
          <Card>
            <CardHeader>
              <CardTitle>Performance Trends</CardTitle>
              <CardDescription>Weekly performance overview</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {engagementTrend.map((week, index) => (
                  <div key={week.week} className="flex items-center gap-3">
                    <div className="w-24 text-sm text-gray-600 dark:text-gray-400">
                      {week.week}
                    </div>
                    <div className="flex-1">
                      <Progress value={week.score} className="h-3" />
                    </div>
                    <div className="w-16 text-right text-sm font-semibold text-gray-900 dark:text-white">
                      {week.score}%
                    </div>
                    {index > 0 && (
                      <div className="w-12">
                        {week.score > engagementTrend[index - 1].score ? (
                          <div className="flex items-center gap-1 text-xs text-green-600">
                            <TrendingUp className="h-3 w-3" />
                            {week.score - engagementTrend[index - 1].score}%
                          </div>
                        ) : (
                          <div className="flex items-center gap-1 text-xs text-red-600">
                            <TrendingDown className="h-3 w-3" />
                            {engagementTrend[index - 1].score - week.score}%
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="engagement" className="space-y-6 mt-6">
          <div className="grid gap-6 md:grid-cols-3">
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Login Frequency</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                  4.2<span className="text-lg text-gray-500">/week</span>
                </div>
                <p className="text-xs text-gray-500">Average logins per student</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Assignment Completion</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                  87%
                </div>
                <p className="text-xs text-gray-500">On-time submission rate</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Discussion Posts</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                  156
                </div>
                <p className="text-xs text-gray-500">This week across all courses</p>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Engagement Score by Week</CardTitle>
              <CardDescription>Student participation and activity levels</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {engagementTrend.map((week) => (
                  <div key={week.week}>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {week.week}
                      </span>
                      <span className="text-sm font-bold text-gray-900 dark:text-white">
                        {week.score}%
                      </span>
                    </div>
                    <Progress value={week.score} className="h-2" />
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="by-course" className="space-y-6 mt-6">
          <div className="grid gap-6">
            {mockCourses.map((course) => (
              <Card key={course.id}>
                <CardHeader>
                  <CardTitle className="flex items-center justify-between">
                    <span>{course.code}: {course.name}</span>
                    <Badge>{course.studentCount} students</Badge>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid gap-6 md:grid-cols-3">
                    <div>
                      <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                        Average Grade
                      </p>
                      <div className="text-3xl font-bold text-gray-900 dark:text-white">
                        {course.averageGrade}%
                      </div>
                    </div>
                    <div>
                      <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                        Active Students
                      </p>
                      <div className="text-3xl font-bold text-gray-900 dark:text-white">
                        {course.activeStudents}
                      </div>
                      <p className="text-xs text-gray-500 mt-1">
                        {Math.round((course.activeStudents / course.studentCount) * 100)}% engagement
                      </p>
                    </div>
                    <div>
                      <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                        Course Progress
                      </p>
                      <div className="text-3xl font-bold text-gray-900 dark:text-white">
                        {course.progress}%
                      </div>
                    </div>
                  </div>
                  <div className="mt-4">
                    <Progress value={course.progress} className="h-2" />
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
