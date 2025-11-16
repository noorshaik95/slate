'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  FileText,
  Clock,
  AlertCircle,
  Filter,
  Download,
  ArrowUpDown,
} from 'lucide-react';
import { mockGradingQueue, mockCourses } from '@/lib/mock-data';
import { formatRelativeTime } from '@/lib/utils';
import { cn } from '@/lib/utils';

const priorityColors = {
  low: 'default' as const,
  medium: 'warning' as const,
  high: 'destructive' as const,
};

const typeIcons = {
  essay: FileText,
  quiz: FileText,
  project: FileText,
  assignment: FileText,
};

export default function GradingPage() {
  const overdueItems = mockGradingQueue.filter(
    (item) => new Date(item.dueDate) < new Date()
  );
  const upcomingItems = mockGradingQueue.filter(
    (item) => new Date(item.dueDate) >= new Date()
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
            Grading
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">
            Grade student assignments and provide feedback
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline">
            <Filter className="h-4 w-4" />
            Filter
          </Button>
          <Button variant="outline">
            <ArrowUpDown className="h-4 w-4" />
            Sort
          </Button>
          <Button variant="outline">
            <Download className="h-4 w-4" />
            Export
          </Button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-600 dark:text-gray-400">
              Total to Grade
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              {mockGradingQueue.length}
            </div>
            <p className="text-xs text-gray-500 mt-1">Across all courses</p>
          </CardContent>
        </Card>

        <Card className="border-red-200 dark:border-red-800">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-red-600 dark:text-red-400">
              Overdue
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-red-600 dark:text-red-400">
              {overdueItems.length}
            </div>
            <p className="text-xs text-gray-500 mt-1">Needs immediate attention</p>
          </CardContent>
        </Card>

        <Card className="border-blue-200 dark:border-blue-800">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-blue-600 dark:text-blue-400">
              Upcoming
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-blue-600 dark:text-blue-400">
              {upcomingItems.length}
            </div>
            <p className="text-xs text-gray-500 mt-1">Due soon</p>
          </CardContent>
        </Card>

        <Card className="border-green-200 dark:border-green-800">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-green-600 dark:text-green-400">
              Graded Today
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-green-600 dark:text-green-400">
              12
            </div>
            <p className="text-xs text-gray-500 mt-1">Great progress!</p>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="all" className="w-full">
        <TabsList>
          <TabsTrigger value="all">
            All ({mockGradingQueue.length})
          </TabsTrigger>
          <TabsTrigger value="overdue">
            Overdue ({overdueItems.length})
          </TabsTrigger>
          <TabsTrigger value="high-priority">
            High Priority (2)
          </TabsTrigger>
          <TabsTrigger value="by-course">
            By Course
          </TabsTrigger>
        </TabsList>

        <TabsContent value="all" className="space-y-4 mt-6">
          <div className="space-y-3">
            {mockGradingQueue.map((item) => {
              const Icon = typeIcons[item.assignmentType];
              const isOverdue = new Date(item.dueDate) < new Date();

              return (
                <Card
                  key={item.id}
                  className={cn(
                    'hover:shadow-md transition-shadow',
                    isOverdue && 'border-red-300 bg-red-50/50 dark:bg-red-950/20'
                  )}
                >
                  <CardContent className="p-4">
                    <div className="flex items-start gap-4">
                      {/* Avatar */}
                      <img
                        src={item.studentAvatar}
                        alt={item.studentName}
                        className="w-12 h-12 rounded-full"
                      />

                      {/* Content */}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-start justify-between mb-2">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-1">
                              <h4 className="font-semibold text-gray-900 dark:text-white">
                                {item.studentName}
                              </h4>
                              <Badge variant={priorityColors[item.priority]} className="text-xs">
                                {item.priority}
                              </Badge>
                            </div>
                            <div className="flex items-center gap-2">
                              <Icon className="w-4 h-4 text-gray-500" />
                              <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                                {item.assignmentName}
                              </p>
                            </div>
                          </div>
                          <div className="text-right">
                            <p className="text-sm font-medium text-gray-900 dark:text-white">
                              {item.points} pts
                            </p>
                          </div>
                        </div>

                        <div className="flex items-center gap-4 text-xs text-gray-500 mb-3">
                          <Badge variant="outline">{item.courseName}</Badge>
                          <span className="flex items-center gap-1">
                            <Clock className="w-3 h-3" />
                            Submitted {formatRelativeTime(item.submittedAt)}
                          </span>
                          {isOverdue && (
                            <span className="flex items-center gap-1 text-red-600 font-medium">
                              <AlertCircle className="w-3 h-3" />
                              Overdue
                            </span>
                          )}
                        </div>

                        <div className="flex gap-2">
                          <Button size="sm" className="flex-1">
                            Grade Now
                          </Button>
                          <Button size="sm" variant="outline">
                            View Submission
                          </Button>
                          <Button size="sm" variant="ghost">
                            Skip
                          </Button>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </TabsContent>

        <TabsContent value="overdue" className="mt-6">
          <div className="space-y-3">
            {overdueItems.map((item) => {
              const Icon = typeIcons[item.assignmentType];

              return (
                <Card
                  key={item.id}
                  className="border-red-300 bg-red-50/50 dark:bg-red-950/20 hover:shadow-md transition-shadow"
                >
                  <CardContent className="p-4">
                    <div className="flex items-start gap-4">
                      <img
                        src={item.studentAvatar}
                        alt={item.studentName}
                        className="w-12 h-12 rounded-full"
                      />
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <h4 className="font-semibold text-gray-900 dark:text-white">
                            {item.studentName}
                          </h4>
                          <Badge variant="destructive" className="text-xs">
                            Overdue
                          </Badge>
                        </div>
                        <div className="flex items-center gap-2 mb-2">
                          <Icon className="w-4 h-4 text-gray-500" />
                          <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                            {item.assignmentName}
                          </p>
                        </div>
                        <div className="flex gap-2">
                          <Button size="sm">Grade Now</Button>
                          <Button size="sm" variant="outline">
                            View Submission
                          </Button>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </TabsContent>

        <TabsContent value="high-priority" className="mt-6">
          <div className="text-center py-12 text-gray-500">
            High priority items will appear here
          </div>
        </TabsContent>

        <TabsContent value="by-course" className="mt-6">
          <div className="space-y-6">
            {mockCourses.map((course) => {
              const courseItems = mockGradingQueue.filter(
                (item) => item.courseId === course.id
              );

              if (courseItems.length === 0) return null;

              return (
                <Card key={course.id}>
                  <CardHeader>
                    <CardTitle className="flex items-center justify-between">
                      <span>{course.code}: {course.name}</span>
                      <Badge>{courseItems.length} to grade</Badge>
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-2">
                    {courseItems.map((item) => (
                      <div
                        key={item.id}
                        className="flex items-center justify-between p-3 border rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800"
                      >
                        <div className="flex items-center gap-3">
                          <img
                            src={item.studentAvatar}
                            alt={item.studentName}
                            className="w-8 h-8 rounded-full"
                          />
                          <div>
                            <p className="font-medium text-sm">{item.studentName}</p>
                            <p className="text-xs text-gray-500">{item.assignmentName}</p>
                          </div>
                        </div>
                        <Button size="sm">Grade</Button>
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
