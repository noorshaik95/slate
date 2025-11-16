'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Calendar, FileText } from 'lucide-react';
import { mockAssignments, mockCourses } from '@/lib/mock-data';
import { AssignmentCard } from '@/components/common/assignment-card';
import { GradientType } from '@/components/common/gradient-card';

// Map course colors to gradients
const colorToGradient: Record<string, GradientType> = {
  blue: 'blue-cyan',
  purple: 'purple-pink',
  green: 'emerald-teal',
  red: 'orange-red',
  orange: 'amber-yellow',
  violet: 'violet-indigo',
  indigo: 'indigo-purple',
};

export default function AssignmentsPage() {
  const router = useRouter();

  // Convert mock assignments to AssignmentCard format
  const assignments = mockAssignments.map((assignment) => {
    const course = mockCourses.find((c) => c.id === assignment.courseId);
    const gradient = course ? colorToGradient[course.color] || 'blue-cyan' : 'blue-cyan';

    return {
      id: assignment.id,
      title: assignment.title,
      description: assignment.description,
      course: {
        code: assignment.courseName,
        gradient,
      },
      points: assignment.points,
      dueDate: new Date(assignment.dueDate),
      type: assignment.submissionType as 'file' | 'text' | 'quiz' | 'exam',
      status: assignment.status,
    };
  });

  const pendingAssignments = assignments.filter((a) => a.status === 'pending');
  const submittedAssignments = assignments.filter((a) => a.status === 'submitted');
  const gradedAssignments = assignments.filter((a) => a.status === 'graded');

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900">
      <div className="space-y-6 p-6 max-w-7xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
              Assignments
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mt-1">
              {pendingAssignments.length} pending assignment{pendingAssignments.length !== 1 ? 's' : ''}
            </p>
          </div>
          <Button
            onClick={() => router.push('/calendar')}
            className="gradient-indigo-purple text-white font-semibold hover-lift"
          >
            <Calendar className="mr-2 h-4 w-4" />
            View Calendar
          </Button>
        </div>

        {/* Pending Assignments */}
        {pendingAssignments.length > 0 && (
          <div className="space-y-4">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">Pending</h2>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {pendingAssignments.map((assignment) => (
                <Link key={assignment.id} href={`/assignments/${assignment.id}`}>
                  <AssignmentCard
                    assignment={assignment}
                    onSubmit={() => router.push(`/assignments/${assignment.id}`)}
                    className="hover-lift"
                  />
                </Link>
              ))}
            </div>
          </div>
        )}

        {/* Submitted Assignments */}
        {submittedAssignments.length > 0 && (
          <div className="space-y-4">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">Submitted</h2>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {submittedAssignments.map((assignment) => (
                <Link key={assignment.id} href={`/assignments/${assignment.id}`}>
                  <AssignmentCard
                    assignment={assignment}
                    className="hover-lift"
                  />
                </Link>
              ))}
            </div>
          </div>
        )}

        {/* Graded Assignments */}
        {gradedAssignments.length > 0 && (
          <div className="space-y-4">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">Graded</h2>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {gradedAssignments.map((assignment) => (
                <Link key={assignment.id} href={`/assignments/${assignment.id}`}>
                  <AssignmentCard
                    assignment={assignment}
                    className="hover-lift"
                  />
                </Link>
              ))}
            </div>
          </div>
        )}

        {/* Empty State */}
        {assignments.length === 0 && (
          <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 p-12">
            <div className="flex flex-col items-center text-center space-y-4">
              <FileText className="h-16 w-16 text-gray-400 dark:text-gray-600" />
              <div className="space-y-2">
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white">
                  No Assignments
                </h3>
                <p className="text-gray-600 dark:text-gray-400">
                  You don&apos;t have any assignments at the moment.
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
