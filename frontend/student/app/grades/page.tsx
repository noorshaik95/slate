import { Button } from '@/components/ui/button';
import { BarChart3, BookOpen, Download, GraduationCap } from 'lucide-react';
import { mockGrades, mockCourses } from '@/lib/mock-data';
import { StatCard } from '@/components/common/stat-card';
import { GradeCard } from '@/components/common/grade-card';
import { GradientType } from '@/components/common/gradient-card';

export const metadata = {
  title: 'Grades | Student Portal',
  description: 'View your grades and academic performance',
};

// Map courses to gradients
const courseGradients: Record<string, GradientType> = {
  '1': 'blue-cyan',
  '2': 'purple-pink',
  '3': 'emerald-teal',
  '4': 'orange-red',
};

export default function GradesPage() {
  // Calculate GPA (simplified)
  const totalPoints = mockGrades.reduce((sum, g) => sum + g.percentage, 0);
  const gpa = (totalPoints / mockGrades.length / 25).toFixed(2); // Simplified GPA calculation
  const totalCredits = mockCourses.reduce((sum, c) => sum + c.credits, 0);
  const averageGrade = Math.round(totalPoints / mockGrades.length);

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900 p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">Grades</h1>
          <p className="text-gray-600 dark:text-gray-400">Track your academic performance</p>
        </div>
        <Button 
          variant="outline" 
          className="bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-all duration-300 hover:-translate-y-1 hover:shadow-lg"
        >
          <Download className="mr-2 h-4 w-4" />
          Export Transcript
        </Button>
      </div>

      {/* Grade Statistics */}
      <div className="grid gap-6 md:grid-cols-3">
        <StatCard
          icon={<GraduationCap className="h-6 w-6" />}
          label="Current GPA"
          value={gpa}
          subtitle="+0.2 from last semester"
          gradient="blue-cyan"
          variant="outline"
        />
        <StatCard
          icon={<BookOpen className="h-6 w-6" />}
          label="Total Credits"
          value={totalCredits}
          subtitle={`Enrolled in ${mockCourses.length} courses`}
          gradient="purple-pink"
          variant="outline"
        />
        <StatCard
          icon={<BarChart3 className="h-6 w-6" />}
          label="Average Grade"
          value={`${averageGrade}%`}
          subtitle={`Across ${mockGrades.length} graded assignments`}
          gradient="emerald-teal"
          variant="outline"
        />
      </div>

      {/* Grades by Course */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white">Grades by Course</h2>
        {mockCourses.map((course) => {
          const courseGrades = mockGrades.filter(g => g.courseId === course.id);
          const courseAvg = courseGrades.length > 0
            ? courseGrades.reduce((sum, g) => sum + g.percentage, 0) / courseGrades.length
            : 0;

          // Map grades to the format expected by GradeCard
          const assignments = courseGrades.map(grade => ({
            id: grade.id,
            name: grade.assignmentName,
            type: grade.category,
            grade: grade.score,
            maxPoints: grade.maxScore,
            date: new Date(grade.gradedAt),
            feedback: grade.feedback,
          }));

          return (
            <GradeCard
              key={course.id}
              course={{
                id: course.id,
                code: course.code,
                name: course.name,
                instructor: course.instructor,
                gradient: courseGradients[course.id] || 'blue-cyan',
                grade: Math.round(courseAvg),
                progress: course.progress,
              }}
              assignments={assignments}
            />
          );
        })}
      </div>
    </div>
  );
}
