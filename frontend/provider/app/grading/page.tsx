import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { FileText } from 'lucide-react';

export const metadata = {
  title: 'Grading | Instructor Portal',
  description: 'Grade student submissions',
};

export default function GradingPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Grading
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Grade student assignments and provide feedback
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            Grading Interface
          </CardTitle>
          <CardDescription>
            This page is under development. Grading features will be available soon.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-gray-600 dark:text-gray-400">
            Coming soon: SpeedGrader, rubric grading, and bulk grading tools.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
