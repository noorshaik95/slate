import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { BarChart3 } from 'lucide-react';

export const metadata = {
  title: 'Analytics | Instructor Portal',
  description: 'View course and student analytics',
};

export default function AnalyticsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Analytics
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Track student performance and course metrics
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BarChart3 className="h-5 w-5" />
            Analytics Dashboard
          </CardTitle>
          <CardDescription>
            This page is under development. Analytics features will be available soon.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-gray-600 dark:text-gray-400">
            Coming soon: Student analytics, engagement metrics, and performance insights.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
