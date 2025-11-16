import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { BookOpen } from 'lucide-react';

export const metadata = {
  title: 'Courses | Instructor Portal',
  description: 'Manage your courses',
};

export default function CoursesPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Courses
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Manage all your courses in one place
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BookOpen className="h-5 w-5" />
            Course Management
          </CardTitle>
          <CardDescription>
            This page is under development. Course management features will be available soon.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-gray-600 dark:text-gray-400">
            Coming soon: Create, edit, and manage your courses.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
