import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Calendar as CalendarIcon } from 'lucide-react';

export const metadata = {
  title: 'Calendar | Instructor Portal',
  description: 'View your schedule',
};

export default function CalendarPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Calendar
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Manage your schedule and upcoming events
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CalendarIcon className="h-5 w-5" />
            Calendar View
          </CardTitle>
          <CardDescription>
            This page is under development. Calendar features will be available soon.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-gray-600 dark:text-gray-400">
            Coming soon: Interactive calendar with classes, office hours, and deadlines.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
