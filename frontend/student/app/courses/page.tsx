import Link from 'next/link';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Button } from '@/components/ui/button';
import { BookOpen, Clock, Users, ArrowRight } from 'lucide-react';
import { mockCourses } from '@/lib/mock-data';

export const metadata = {
  title: 'My Courses | Student Portal',
  description: 'View and manage your enrolled courses',
};

export default function CoursesPage() {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">My Courses</h1>
        <p className="text-muted-foreground">
          You are enrolled in {mockCourses.length} courses this semester
        </p>
      </div>

      {/* Course Grid */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {mockCourses.map((course) => (
          <Link key={course.id} href={`/courses/${course.id}`} className="group">
            <Card className="h-full transition-all hover:shadow-lg">
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <Badge variant="secondary" className="mb-2">
                      {course.code}
                    </Badge>
                    <CardTitle className="group-hover:text-primary">
                      {course.name}
                    </CardTitle>
                    <CardDescription className="line-clamp-2">
                      {course.description}
                    </CardDescription>
                  </div>
                  <div className={`flex h-10 w-10 items-center justify-center rounded-full bg-${course.color}-100 dark:bg-${course.color}-900`}>
                    <BookOpen className={`h-5 w-5 text-${course.color}-600 dark:text-${course.color}-400`} />
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-4">
                {/* Progress */}
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">Progress</span>
                    <span className="font-medium">{course.progress}%</span>
                  </div>
                  <Progress value={course.progress} className="h-2" />
                </div>

                {/* Stats */}
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div className="flex items-center gap-2 text-muted-foreground">
                    <Users className="h-4 w-4" />
                    <span>{course.enrollmentCount} students</span>
                  </div>
                  <div className="flex items-center gap-2 text-muted-foreground">
                    <Clock className="h-4 w-4" />
                    <span>{course.credits} credits</span>
                  </div>
                </div>

                {/* Instructor */}
                <div className="pt-2 border-t">
                  <p className="text-sm text-muted-foreground">Instructor</p>
                  <p className="text-sm font-medium">{course.instructor}</p>
                </div>

                {/* Next Deadline */}
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Next deadline</span>
                  <Badge variant="outline">{course.nextDeadline}</Badge>
                </div>

                {/* Action */}
                <Button variant="ghost" className="w-full group-hover:bg-primary group-hover:text-primary-foreground">
                  View Course
                  <ArrowRight className="ml-2 h-4 w-4" />
                </Button>
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>

      {/* Empty State (if no courses) */}
      {mockCourses.length === 0 && (
        <Card className="p-12">
          <div className="flex flex-col items-center text-center space-y-4">
            <BookOpen className="h-16 w-16 text-muted-foreground/50" />
            <div className="space-y-2">
              <h3 className="text-xl font-semibold">No Courses Yet</h3>
              <p className="text-muted-foreground">
                You haven&apos;t enrolled in any courses. Contact your advisor to get started.
              </p>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
}
