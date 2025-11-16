'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Mail,
  Download,
  Filter,
  CheckCircle,
  Clock,
} from 'lucide-react';
import { mockStudents, mockCourses } from '@/lib/mock-data';
import { formatRelativeTime } from '@/lib/utils';

export default function StudentsPage() {
  const activeStudents = mockStudents.filter((s) => s.status === 'active');
  const atRiskStudents = mockStudents.filter((s) => s.status === 'at-risk');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
            Students
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">
            View and manage all your students
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline">
            <Filter className="h-4 w-4" />
            Filter
          </Button>
          <Button variant="outline">
            <Download className="h-4 w-4" />
            Export
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-600 dark:text-gray-400">
              Total Students
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              {mockStudents.length}
            </div>
          </CardContent>
        </Card>

        <Card className="border-green-200 dark:border-green-800">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-green-600 dark:text-green-400">
              Active
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-green-600 dark:text-green-400">
              {activeStudents.length}
            </div>
          </CardContent>
        </Card>

        <Card className="border-orange-200 dark:border-orange-800">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-orange-600 dark:text-orange-400">
              At-Risk
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-orange-600 dark:text-orange-400">
              {atRiskStudents.length}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-600 dark:text-gray-400">
              Avg. Grade
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              82%
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="all" className="w-full">
        <TabsList>
          <TabsTrigger value="all">All Students ({mockStudents.length})</TabsTrigger>
          <TabsTrigger value="active">Active ({activeStudents.length})</TabsTrigger>
          <TabsTrigger value="at-risk">At-Risk ({atRiskStudents.length})</TabsTrigger>
          <TabsTrigger value="by-course">By Course</TabsTrigger>
        </TabsList>

        <TabsContent value="all" className="space-y-4 mt-6">
          <div className="space-y-3">
            {mockStudents.map((student) => (
              <Card key={student.id} className="hover:shadow-md transition-shadow">
                <CardContent className="p-4">
                  <div className="flex items-start gap-4">
                    <img
                      src={student.avatar}
                      alt={student.name}
                      className="w-12 h-12 rounded-full"
                    />
                    <div className="flex-1">
                      <div className="flex items-start justify-between mb-2">
                        <div>
                          <div className="flex items-center gap-2 mb-1">
                            <h4 className="font-semibold text-gray-900 dark:text-white">
                              {student.name}
                            </h4>
                            {student.status === 'at-risk' && (
                              <Badge variant="destructive" className="text-xs">
                                At-Risk
                              </Badge>
                            )}
                          </div>
                          <p className="text-sm text-gray-600 dark:text-gray-400">
                            {student.email}
                          </p>
                        </div>
                        <div className="text-right">
                          <div className="text-2xl font-bold text-gray-900 dark:text-white">
                            {student.currentGrade}%
                          </div>
                          <p className="text-xs text-gray-500">Current Grade</p>
                        </div>
                      </div>

                      <div className="flex items-center gap-4 mb-3 text-sm">
                        <Badge variant="outline">{student.courseName}</Badge>
                        <span className="text-gray-600 dark:text-gray-400">
                          {student.assignmentsCompleted}/{student.assignmentsTotal} assignments
                        </span>
                        <span className="flex items-center gap-1 text-gray-500">
                          <Clock className="h-3 w-3" />
                          Active {formatRelativeTime(student.lastActive)}
                        </span>
                      </div>

                      <div className="mb-3">
                        <div className="flex items-center justify-between mb-1">
                          <span className="text-xs text-gray-600 dark:text-gray-400">
                            Progress
                          </span>
                          <span className="text-xs font-semibold">
                            {Math.round((student.assignmentsCompleted / student.assignmentsTotal) * 100)}%
                          </span>
                        </div>
                        <Progress
                          value={(student.assignmentsCompleted / student.assignmentsTotal) * 100}
                          className="h-2"
                        />
                      </div>

                      <div className="flex gap-2">
                        <Button size="sm" variant="outline">
                          <Mail className="h-3 w-3" />
                          Email
                        </Button>
                        <Button size="sm" variant="outline">
                          View Profile
                        </Button>
                        <Button size="sm" variant="outline">
                          View Grades
                        </Button>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="active" className="mt-6">
          <div className="space-y-3">
            {activeStudents.map((student) => (
              <Card key={student.id}>
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <img
                        src={student.avatar}
                        alt={student.name}
                        className="w-10 h-10 rounded-full"
                      />
                      <div>
                        <h4 className="font-semibold text-sm">{student.name}</h4>
                        <p className="text-xs text-gray-500">{student.courseName}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-4">
                      <div className="text-right">
                        <div className="font-bold text-green-600">{student.currentGrade}%</div>
                        <p className="text-xs text-gray-500">Grade</p>
                      </div>
                      <CheckCircle className="h-5 w-5 text-green-600" />
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="at-risk" className="mt-6">
          <div className="space-y-3">
            {atRiskStudents.map((student) => (
              <Card
                key={student.id}
                className="border-orange-200 dark:border-orange-800 bg-orange-50 dark:bg-orange-950/20"
              >
                <CardContent className="p-4">
                  <div className="flex items-start gap-3">
                    <img
                      src={student.avatar}
                      alt={student.name}
                      className="w-10 h-10 rounded-full"
                    />
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <h4 className="font-semibold text-sm">{student.name}</h4>
                        <Badge variant="destructive" className="text-xs">At-Risk</Badge>
                      </div>
                      <p className="text-xs text-gray-600 dark:text-gray-400 mb-2">
                        {student.courseName} • {student.currentGrade}% • {student.assignmentsTotal - student.assignmentsCompleted} assignments missing
                      </p>
                      <div className="flex gap-2">
                        <Button size="sm">Contact Student</Button>
                        <Button size="sm" variant="outline">View Details</Button>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="by-course" className="mt-6">
          <div className="space-y-6">
            {mockCourses.map((course) => {
              const courseStudents = mockStudents.filter((s) => s.courseId === course.id);
              return (
                <Card key={course.id}>
                  <CardHeader>
                    <CardTitle className="flex items-center justify-between">
                      <span>{course.code}: {course.name}</span>
                      <Badge>{courseStudents.length} students</Badge>
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-2">
                    {courseStudents.map((student) => (
                      <div
                        key={student.id}
                        className="flex items-center justify-between p-3 border rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800"
                      >
                        <div className="flex items-center gap-3">
                          <img
                            src={student.avatar}
                            alt={student.name}
                            className="w-8 h-8 rounded-full"
                          />
                          <div>
                            <p className="font-medium text-sm">{student.name}</p>
                            <p className="text-xs text-gray-500">{student.email}</p>
                          </div>
                        </div>
                        <div className="text-right">
                          <div className="font-bold">{student.currentGrade}%</div>
                          <p className="text-xs text-gray-500">
                            {student.assignmentsCompleted}/{student.assignmentsTotal}
                          </p>
                        </div>
                      </div>
                    ))}
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
