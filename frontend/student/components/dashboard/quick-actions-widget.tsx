'use client';

import { Upload, MessageSquare, Calendar, BookOpen, FileText, Video } from 'lucide-react';
import { QuickActionButton } from '@/components/common/quick-action-button';
import { GradientType } from '@/components/common/gradient-card';

const quickActions = [
  {
    name: 'Submit Work',
    icon: <Upload className="h-6 w-6" />,
    href: '/assignments',
    gradient: 'blue-cyan' as GradientType,
  },
  {
    name: 'Discussions',
    icon: <MessageSquare className="h-6 w-6" />,
    href: '/discussions',
    gradient: 'emerald-teal' as GradientType,
  },
  {
    name: 'Schedule',
    icon: <Calendar className="h-6 w-6" />,
    href: '/calendar',
    gradient: 'purple-pink' as GradientType,
  },
  {
    name: 'My Courses',
    icon: <BookOpen className="h-6 w-6" />,
    href: '/courses',
    gradient: 'orange-red' as GradientType,
  },
  {
    name: 'View Grades',
    icon: <FileText className="h-6 w-6" />,
    href: '/grades',
    gradient: 'amber-yellow' as GradientType,
  },
  {
    name: 'Live Classes',
    icon: <Video className="h-6 w-6" />,
    href: '/video',
    gradient: 'violet-indigo' as GradientType,
  },
];

export function QuickActionsWidget() {
  return (
    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-6">
      {quickActions.map((action) => (
        <QuickActionButton
          key={action.name}
          icon={action.icon}
          label={action.name}
          gradient={action.gradient}
          href={action.href}
        />
      ))}
    </div>
  );
}
