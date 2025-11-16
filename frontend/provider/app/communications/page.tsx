import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { MessagesSquare } from 'lucide-react';

export const metadata = {
  title: 'Communications | Instructor Portal',
  description: 'Communicate with your students',
};

export default function CommunicationsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Communications
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Announcements, messages, and discussions
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <MessagesSquare className="h-5 w-5" />
            Communication Center
          </CardTitle>
          <CardDescription>
            This page is under development. Communication features will be available soon.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-gray-600 dark:text-gray-400">
            Coming soon: Announcements, messages, and discussion moderation.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
