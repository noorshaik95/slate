'use client';

import { Card, CardContent } from '@/components/ui/card';
import { Upload, MessageSquare, Calendar, BookOpen, FileText, Video } from 'lucide-react';
import Link from 'next/link';

const quickActions = [
  {
    name: 'Submit Assignment',
    icon: Upload,
    href: '/assignments',
    color: 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300',
  },
  {
    name: 'Join Discussion',
    icon: MessageSquare,
    href: '/discussions',
    color: 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300',
  },
  {
    name: 'View Schedule',
    icon: Calendar,
    href: '/calendar',
    color: 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300',
  },
  {
    name: 'My Courses',
    icon: BookOpen,
    href: '/courses',
    color: 'bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300',
  },
  {
    name: 'View Grades',
    icon: FileText,
    href: '/grades',
    color: 'bg-pink-100 text-pink-700 dark:bg-pink-900 dark:text-pink-300',
  },
  {
    name: 'Live Classes',
    icon: Video,
    href: '/classes',
    color: 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900 dark:text-indigo-300',
  },
];

export function QuickActionsWidget() {
  return (
    <Card>
      <CardContent className="pt-6">
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-6">
          {quickActions.map((action) => (
            <Link
              key={action.name}
              href={action.href}
              className="group flex flex-col items-center gap-2 rounded-lg p-4 transition-colors hover:bg-accent focus-ring"
            >
              <div className={`rounded-lg p-3 ${action.color} transition-transform group-hover:scale-110`}>
                <action.icon className="h-6 w-6" aria-hidden="true" />
              </div>
              <span className="text-center text-sm font-medium">{action.name}</span>
            </Link>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
