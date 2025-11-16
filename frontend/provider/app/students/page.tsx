import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Users } from 'lucide-react';

export const metadata = {
  title: 'Students | Instructor Portal',
  description: 'Manage your students',
};

export default function StudentsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Students
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          View and manage all your students
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            Student Management
          </CardTitle>
          <CardDescription>
            This page is under development. Student management features will be available soon.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-gray-600 dark:text-gray-400">
            Coming soon: Student roster, performance tracking, and communication tools.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
