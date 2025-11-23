'use client';

import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Plus,
  MoreVertical,
  Eye,
  Edit,
  FileText,
  Link as LinkIcon,
  Users,
  TrendingUp,
  Calendar,
  Settings,
} from 'lucide-react';
import { mockCourses } from '@/lib/mock-data';
import { cn } from '@/lib/utils';
import Link from 'next/link';

const gradientClasses: Record<string, string> = {
  'blue-cyan': 'from-blue-500 to-cyan-500',
  'purple-pink': 'from-purple-500 to-pink-500',
  'emerald-teal': 'from-emerald-500 to-teal-500',
  'orange-red': 'from-orange-500 to-red-500',
  'amber-yellow': 'from-amber-500 to-yellow-500',
  'violet-indigo': 'from-violet-500 to-indigo-500',
  'indigo-purple': 'from-indigo-500 to-purple-500',
};

export default function CoursesPage() {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
            Courses
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">
            Manage all your courses in one place
          </p>
        </div>
        <Button size="lg" asChild>
          <Link href="/courses/create">
            <Plus className="h-5 w-5" />
            Create New Course
          </Link>
        </Button>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="all" className="w-full">
        <TabsList>
          <TabsTrigger value="all">All Courses ({mockCourses.length})</TabsTrigger>
          <TabsTrigger value="active">Active (3)</TabsTrigger>
          <TabsTrigger value="archived">Archived (0)</TabsTrigger>
        </TabsList>

        <TabsContent value="all" className="space-y-4 mt-6">
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
            {mockCourses.map((course) => (
              <Card key={course.id} className="hover:shadow-lg transition-shadow">
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <div
                          className={cn(
                            'px-3 py-1 rounded-md text-white text-sm font-semibold bg-gradient-to-r',
                            gradientClasses[course.gradient]
                          )}
                        >
                          {course.code}
                        </div>
                        <Badge variant="outline">{course.section}</Badge>
                      </div>
                      <CardTitle className="text-lg">{course.name}</CardTitle>
                      <CardDescription className="mt-1">
                        {course.semester} {course.year}
                      </CardDescription>
                    </div>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="icon">
                          <MoreVertical className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem>
                          <Eye className="h-4 w-4 mr-2" />
                          View Course
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                          <Edit className="h-4 w-4 mr-2" />
                          Edit Details
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                          <FileText className="h-4 w-4 mr-2" />
                          Manage Content
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                          <LinkIcon className="h-4 w-4 mr-2" />
                          Copy Invite Link
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem>
                          <Settings className="h-4 w-4 mr-2" />
                          Course Settings
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </CardHeader>

                <CardContent>
                  <div className="space-y-4">
                    {/* Stats */}
                    <div className="grid grid-cols-3 gap-3">
                      <div className="text-center p-2 rounded-lg bg-gray-50 dark:bg-gray-800">
                        <div className="flex items-center justify-center gap-1 text-gray-500 dark:text-gray-400 mb-1">
                          <Users className="h-3 w-3" />
                        </div>
                        <p className="text-lg font-bold text-gray-900 dark:text-white">
                          {course.studentCount}
                        </p>
                        <p className="text-xs text-gray-600 dark:text-gray-400">Students</p>
                      </div>
                      <div className="text-center p-2 rounded-lg bg-gray-50 dark:bg-gray-800">
                        <div className="flex items-center justify-center gap-1 text-gray-500 dark:text-gray-400 mb-1">
                          <FileText className="h-3 w-3" />
                        </div>
                        <p className="text-lg font-bold text-gray-900 dark:text-white">
                          {course.pendingGrading}
                        </p>
                        <p className="text-xs text-gray-600 dark:text-gray-400">To Grade</p>
                      </div>
                      <div className="text-center p-2 rounded-lg bg-gray-50 dark:bg-gray-800">
                        <div className="flex items-center justify-center gap-1 text-gray-500 dark:text-gray-400 mb-1">
                          <TrendingUp className="h-3 w-3" />
                        </div>
                        <p className="text-lg font-bold text-gray-900 dark:text-white">
                          {course.averageGrade}%
                        </p>
                        <p className="text-xs text-gray-600 dark:text-gray-400">Avg Grade</p>
                      </div>
                    </div>

                    {/* Progress */}
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm text-gray-600 dark:text-gray-400">
                          Course Progress
                        </span>
                        <span className="text-sm font-semibold text-gray-900 dark:text-white">
                          {course.progress}%
                        </span>
                      </div>
                      <Progress value={course.progress} className="h-2" />
                    </div>

                    {/* Schedule */}
                    <div className="pt-2 border-t">
                      <div className="flex items-center gap-2 text-xs text-gray-600 dark:text-gray-400">
                        <Calendar className="h-3 w-3" />
                        <span>{course.schedule.length} classes per week</span>
                      </div>
                    </div>
                  </div>
                </CardContent>

                <CardFooter className="flex gap-2">
                  <Button variant="outline" size="sm" className="flex-1">
                    Manage
                  </Button>
                  <Button size="sm" className="flex-1">
                    Enter Course
                  </Button>
                </CardFooter>
              </Card>
            ))}

            {/* Create New Course Card */}
            <Card className="border-dashed border-2 hover:border-primary hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors cursor-pointer">
              <CardHeader className="h-full flex items-center justify-center">
                <div className="text-center space-y-3">
                  <div className="mx-auto w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center">
                    <Plus className="h-6 w-6 text-primary" />
                  </div>
                  <div>
                    <CardTitle className="text-lg">Create New Course</CardTitle>
                    <CardDescription className="mt-1">
                      Start a new course from scratch or use a template
                    </CardDescription>
                  </div>
                  <Button asChild>
                    <Link href="/courses/create">Get Started</Link>
                  </Button>
                </div>
              </CardHeader>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="active">
          <div className="text-center py-12 text-gray-500">
            Active courses view - Same as &quot;All&quot; for now
          </div>
        </TabsContent>

        <TabsContent value="archived">
          <div className="text-center py-12 text-gray-500">
            No archived courses
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
