'use client';

import { Bell, Settings as SettingsIcon, Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { mockInstructor } from '@/lib/mock-data';
import Link from 'next/link';

interface HeaderProps {
  className?: string;
}

export function Header({ className }: HeaderProps) {
  return (
    <header className={className}>
      {/* Top Bar */}
      <div className="flex h-16 items-center justify-between border-b px-6 bg-white dark:bg-gray-900">
        {/* Welcome Message */}
        <div className="flex-1">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Welcome back, Prof. {mockInstructor.lastName}!
          </h2>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-3">
          <Button asChild>
            <Link href="/courses/create">
              <Plus className="h-4 w-4" />
              Create Course
            </Link>
          </Button>

          <Button
            variant="ghost"
            size="icon"
            aria-label="Notifications"
            className="relative"
          >
            <Bell className="h-5 w-5" />
            <Badge
              variant="destructive"
              className="absolute -right-1 -top-1 h-5 w-5 rounded-full p-0 flex items-center justify-center text-xs"
            >
              5
            </Badge>
          </Button>

          <Button variant="ghost" size="icon" asChild aria-label="Settings">
            <Link href="/settings">
              <SettingsIcon className="h-5 w-5" />
            </Link>
          </Button>

          {/* User Avatar */}
          <div className="flex items-center gap-2 pl-2 border-l">
            <img
              src={mockInstructor.avatar}
              alt={`${mockInstructor.firstName} ${mockInstructor.lastName}`}
              className="h-8 w-8 rounded-full"
            />
            <div className="hidden md:block text-sm">
              <p className="font-medium text-gray-900 dark:text-white">
                {mockInstructor.firstName} {mockInstructor.lastName}
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                {mockInstructor.department}
              </p>
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}
