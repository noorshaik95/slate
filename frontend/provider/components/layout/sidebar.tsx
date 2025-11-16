'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { cn } from '@/lib/utils';
import {
  LayoutDashboard,
  BookOpen,
  FileText,
  BarChart3,
  Calendar,
  Settings,
  Users,
  GraduationCap,
  MessagesSquare,
} from 'lucide-react';

interface SidebarProps {
  className?: string;
}

const navigation = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { name: 'Courses', href: '/courses', icon: BookOpen },
  { name: 'Grading', href: '/grading', icon: FileText },
  { name: 'Analytics', href: '/analytics', icon: BarChart3 },
  { name: 'Students', href: '/students', icon: Users },
  { name: 'Communications', href: '/communications', icon: MessagesSquare },
  { name: 'Calendar', href: '/calendar', icon: Calendar },
  { name: 'Settings', href: '/settings', icon: Settings },
];

export function Sidebar({ className }: SidebarProps) {
  const pathname = usePathname();

  return (
    <aside
      className={cn(
        'flex flex-col border-r bg-background w-64',
        className
      )}
      role="navigation"
      aria-label="Main navigation"
    >
      {/* Logo/Brand */}
      <div className="flex h-16 items-center justify-center border-b px-4">
        <Link href="/dashboard" className="flex items-center gap-2 font-semibold">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-blue-600 to-cyan-600 text-white">
            <GraduationCap className="h-6 w-6" />
          </div>
          <div className="flex flex-col">
            <span className="text-lg">Instructor Portal</span>
            <span className="text-xs text-muted-foreground">Course Management</span>
          </div>
        </Link>
      </div>

      {/* Navigation Links */}
      <nav className="flex-1 space-y-1 p-4" aria-label="Sidebar navigation">
        {navigation.map((item) => {
          const isActive = pathname === item.href || pathname.startsWith(`${item.href}/`);
          return (
            <Link
              key={item.name}
              href={item.href}
              className={cn(
                'flex items-center gap-3 rounded-lg px-4 py-3 text-sm font-medium transition-all',
                'hover:translate-x-1',
                isActive
                  ? 'bg-gradient-to-r from-blue-600 to-cyan-600 text-white shadow-md'
                  : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'
              )}
              aria-current={isActive ? 'page' : undefined}
            >
              <item.icon className="h-5 w-5 shrink-0" aria-hidden="true" />
              <span>{item.name}</span>
            </Link>
          );
        })}
      </nav>

      {/* Quick Stats */}
      <div className="border-t p-4">
        <div className="rounded-lg bg-gradient-to-br from-purple-50 to-blue-50 dark:from-purple-950/20 dark:to-blue-950/20 p-4">
          <p className="text-xs font-semibold text-gray-700 dark:text-gray-300 mb-2">
            Quick Stats
          </p>
          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">Active Courses</span>
              <span className="font-semibold text-gray-900 dark:text-white">3</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">Total Students</span>
              <span className="font-semibold text-gray-900 dark:text-white">317</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">To Grade</span>
              <span className="font-semibold text-orange-600 dark:text-orange-400">46</span>
            </div>
          </div>
        </div>
      </div>
    </aside>
  );
}
