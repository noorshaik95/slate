import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { BarChart3, TrendingUp, Download, FileText } from 'lucide-react';
import { mockGrades, mockCourses } from '@/lib/mock-data';
import { getGradeColor, getGradeLetter, formatDate } from '@/lib/utils';

export const metadata = {
  title: 'Grades | Student Portal',
  description: 'View your grades and academic performance',
};

export default function GradesPage() {
  // Calculate GPA (simplified)
  const totalPoints = mockGrades.reduce((sum, g) => sum + g.percentage, 0);
  const gpa = (totalPoints / mockGrades.length / 25).toFixed(2); // Simplified GPA calculation

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Grades</h1>
          <p className="text-muted-foreground">Track your academic performance</p>
        </div>
        <Button variant="outline">
          <Download className="mr-2 h-4 w-4" />
          Export Transcript
        </Button>
      </div>

      {/* GPA Overview */}
      <div className="grid gap-6 md:grid-cols-3">
        <Card>
          <CardHeader>
            <CardDescription>Current GPA</CardDescription>
            <CardTitle className="text-4xl">{gpa}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2 text-green-600">
              <TrendingUp className="h-4 w-4" />
              <span className="text-sm">+0.2 from last semester</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardDescription>Total Credits</CardDescription>
            <CardTitle className="text-4xl">
              {mockCourses.reduce((sum, c) => sum + c.credits, 0)}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Enrolled in {mockCourses.length} courses
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardDescription>Average Grade</CardDescription>
            <CardTitle className="text-4xl">
              {Math.round(totalPoints / mockGrades.length)}%
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Across {mockGrades.length} graded assignments
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Grades by Course */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Grades by Course</h2>
        {mockCourses.map((course) => {
          const courseGrades = mockGrades.filter(g => g.courseId === course.id);
          const courseAvg = courseGrades.length > 0
            ? courseGrades.reduce((sum, g) => sum + g.percentage, 0) / courseGrades.length
            : 0;

          return (
            <Card key={course.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <Badge>{course.code}</Badge>
                      {courseAvg > 0 && (
                        <Badge className={getGradeColor(courseAvg)}>
                          {getGradeLetter(courseAvg)}
                        </Badge>
                      )}
                    </div>
                    <CardTitle>{course.name}</CardTitle>
                    <CardDescription>{course.instructor}</CardDescription>
                  </div>
                  <div className="text-right">
                    <p className="text-3xl font-bold">{Math.round(courseAvg)}%</p>
                    <p className="text-sm text-muted-foreground">{courseGrades.length} graded</p>
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-4">
                {/* Progress */}
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">Course Progress</span>
                    <span>{course.progress}%</span>
                  </div>
                  <Progress value={course.progress} className="h-2" />
                </div>

                {/* Individual Grades */}
                {courseGrades.length > 0 && (
                  <div className="space-y-2">
                    {courseGrades.map((grade) => (
                      <div key={grade.id} className="flex items-center justify-between p-3 border rounded-lg">
                        <div className="flex-1">
                          <p className="font-medium">{grade.assignmentName}</p>
                          <div className="flex items-center gap-4 text-sm text-muted-foreground">
                            <span>{grade.category}</span>
                            <span>{formatDate(grade.gradedAt)}</span>
                          </div>
                          {grade.feedback && (
                            <p className="mt-1 text-sm text-muted-foreground">{grade.feedback}</p>
                          )}
                        </div>
                        <div className="text-right ml-4">
                          <p className={`text-xl font-bold ${getGradeColor(grade.percentage)}`}>
                            {grade.percentage}%
                          </p>
                          <p className="text-sm text-muted-foreground">
                            {grade.score}/{grade.maxScore}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {courseGrades.length === 0 && (
                  <div className="flex flex-col items-center justify-center py-8 text-center">
                    <FileText className="mb-2 h-12 w-12 text-muted-foreground/50" />
                    <p className="text-sm text-muted-foreground">No grades yet for this course</p>
                  </div>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
