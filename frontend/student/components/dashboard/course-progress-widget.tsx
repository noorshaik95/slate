'use client';

import Link from 'next/link';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { ArrowRight } from 'lucide-react';
import { Button } from '@/components/ui/button';

const enrolledCourses = [
  {
    id: '1',
    code: 'CS 101',
    name: 'Introduction to Computer Science',
    progress: 75,
    instructor: 'Dr. Smith',
    nextDeadline: '2 days',
  },
  {
    id: '2',
    code: 'MATH 201',
    name: 'Calculus II',
    progress: 60,
    instructor: 'Prof. Johnson',
    nextDeadline: '5 days',
  },
  {
    id: '3',
    code: 'ENG 102',
    name: 'English Composition',
    progress: 90,
    instructor: 'Dr. Williams',
    nextDeadline: '1 day',
  },
  {
    id: '4',
    code: 'PHY 201',
    name: 'Physics I',
    progress: 45,
    instructor: 'Dr. Brown',
    nextDeadline: '3 days',
  },
];

export function CourseProgressWidget() {
  return (
    <Card className="h-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>My Courses</CardTitle>
            <CardDescription>Track your progress across all enrolled courses</CardDescription>
          </div>
          <Button asChild variant="ghost" size="sm">
            <Link href="/courses">
              View All
              <ArrowRight className="ml-2 h-4 w-4" />
            </Link>
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {enrolledCourses.map((course) => (
          <Link
            key={course.id}
            href={`/courses/${course.id}`}
            className="block rounded-lg border p-4 transition-colors hover:bg-accent focus-ring"
          >
            <div className="space-y-2">
              <div className="flex items-start justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-sm font-medium text-muted-foreground">
                      {course.code}
                    </span>
                    <span className="text-xs text-muted-foreground">•</span>
                    <span className="text-sm text-muted-foreground">{course.instructor}</span>
                  </div>
                  <h4 className="font-semibold">{course.name}</h4>
                </div>
                <span className="text-sm font-medium">{course.progress}%</span>
              </div>
              <Progress value={course.progress} className="h-2" />
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span>Next deadline in {course.nextDeadline}</span>
                <span className="text-primary">Continue →</span>
              </div>
            </div>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
