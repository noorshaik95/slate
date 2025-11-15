import Link from 'next/link';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Clock, FileText, CheckCircle2, AlertCircle, Calendar } from 'lucide-react';
import { mockAssignments } from '@/lib/mock-data';
import { formatRelativeTime } from '@/lib/utils';

export const metadata = {
  title: 'Assignments | Student Portal',
  description: 'View and manage your assignments',
};

export default function AssignmentsPage() {
  const pendingAssignments = mockAssignments.filter(a => a.status === 'pending');
  const submittedAssignments = mockAssignments.filter(a => a.status === 'submitted' || a.status === 'graded');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Assignments</h1>
          <p className="text-muted-foreground">
            {pendingAssignments.length} pending assignments
          </p>
        </div>
        <Button variant="outline">
          <Calendar className="mr-2 h-4 w-4" />
          View Calendar
        </Button>
      </div>

      {/* Pending Assignments */}
      {pendingAssignments.length > 0 && (
        <div className="space-y-4">
          <h2 className="text-xl font-semibold">Pending</h2>
          <div className="grid gap-4">
            {pendingAssignments.map((assignment) => {
              const daysUntilDue = Math.floor(
                (new Date(assignment.dueDate).getTime() - Date.now()) / (24 * 60 * 60 * 1000)
              );
              const isUrgent = daysUntilDue <= 2;

              return (
                <Link key={assignment.id} href={`/assignments/${assignment.id}`}>
                  <Card className="transition-all hover:shadow-md">
                    <CardHeader>
                      <div className="flex items-start justify-between">
                        <div className="space-y-1">
                          <div className="flex items-center gap-2">
                            <Badge variant="secondary">{assignment.courseName}</Badge>
                            <Badge variant={isUrgent ? 'destructive' : 'default'}>
                              {assignment.points} points
                            </Badge>
                          </div>
                          <CardTitle className="hover:text-primary">{assignment.title}</CardTitle>
                          <CardDescription>{assignment.description}</CardDescription>
                        </div>
                        <AlertCircle className={`h-5 w-5 ${isUrgent ? 'text-red-600' : 'text-orange-600'}`} />
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <div className="flex items-center gap-1">
                          <Clock className="h-4 w-4" />
                          <span className={isUrgent ? 'text-red-600 font-medium' : ''}>
                            Due {formatRelativeTime(assignment.dueDate)}
                          </span>
                        </div>
                        <div className="flex items-center gap-1">
                          <FileText className="h-4 w-4" />
                          <span>{assignment.submissionType}</span>
                        </div>
                      </div>
                      <Button className="w-full">Submit Assignment</Button>
                    </CardContent>
                  </Card>
                </Link>
              );
            })}
          </div>
        </div>
      )}

      {/* Submitted Assignments */}
      {submittedAssignments.length > 0 && (
        <div className="space-y-4">
          <h2 className="text-xl font-semibold">Submitted</h2>
          <div className="grid gap-4">
            {submittedAssignments.map((assignment) => (
              <Link key={assignment.id} href={`/assignments/${assignment.id}`}>
                <Card className="transition-all hover:shadow-md">
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div className="space-y-1">
                        <div className="flex items-center gap-2">
                          <Badge variant="secondary">{assignment.courseName}</Badge>
                          <Badge variant="success">{assignment.status}</Badge>
                        </div>
                        <CardTitle className="hover:text-primary">{assignment.title}</CardTitle>
                        <CardDescription>{assignment.description}</CardDescription>
                      </div>
                      <CheckCircle2 className="h-5 w-5 text-green-600" />
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="flex items-center gap-4 text-sm text-muted-foreground">
                      {assignment.submittedAt && (
                        <span>Submitted {formatRelativeTime(assignment.submittedAt)}</span>
                      )}
                      <span>{assignment.points} points</span>
                    </div>
                  </CardContent>
                </Card>
              </Link>
            ))}
          </div>
        </div>
      )}

      {/* Empty State */}
      {mockAssignments.length === 0 && (
        <Card className="p-12">
          <div className="flex flex-col items-center text-center space-y-4">
            <FileText className="h-16 w-16 text-muted-foreground/50" />
            <div className="space-y-2">
              <h3 className="text-xl font-semibold">No Assignments</h3>
              <p className="text-muted-foreground">
                You don&apos;t have any assignments at the moment.
              </p>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
}
