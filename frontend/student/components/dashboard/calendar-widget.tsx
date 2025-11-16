'use client';

import Link from 'next/link';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Calendar as CalendarIcon, Clock } from 'lucide-react';
import { formatDate } from '@/lib/utils';

const todayEvents = [
  {
    id: '1',
    title: 'CS 101 Lecture',
    type: 'class' as const,
    startTime: '09:00 AM',
    endTime: '10:30 AM',
    location: 'Room 301',
  },
  {
    id: '2',
    title: 'MATH 201 Tutorial',
    type: 'class' as const,
    startTime: '02:00 PM',
    endTime: '03:00 PM',
    location: 'Room 205',
  },
  {
    id: '3',
    title: 'Assignment Due: Physics Lab',
    type: 'deadline' as const,
    startTime: '11:59 PM',
    endTime: '11:59 PM',
    location: 'Online',
  },
];

const upcomingEvents = [
  {
    id: '4',
    title: 'Study Group',
    date: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    time: '03:00 PM',
    type: 'event' as const,
  },
  {
    id: '5',
    title: 'Office Hours - Dr. Smith',
    date: new Date(Date.now() + 2 * 24 * 60 * 60 * 1000).toISOString(),
    time: '10:00 AM',
    type: 'event' as const,
  },
];

export function CalendarWidget() {
  const getEventColor = (type: 'class' | 'deadline' | 'event' | 'exam') => {
    switch (type) {
      case 'class':
        return 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300';
      case 'deadline':
        return 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300';
      case 'exam':
        return 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300';
      default:
        return 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300';
    }
  };

  return (
    <Card className="h-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Today&apos;s Schedule</CardTitle>
            <CardDescription>{formatDate(new Date())}</CardDescription>
          </div>
          <Button asChild variant="ghost" size="icon">
            <Link href="/calendar" aria-label="View full calendar">
              <CalendarIcon className="h-5 w-5" />
            </Link>
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Today's Events */}
        <div className="space-y-2">
          <h4 className="text-sm font-semibold">Today</h4>
          {todayEvents.length === 0 ? (
            <p className="text-sm text-muted-foreground">No events scheduled</p>
          ) : (
            <div className="space-y-2">
              {todayEvents.map((event) => (
                <div
                  key={event.id}
                  className="flex items-start gap-3 rounded-lg border p-2 transition-colors hover:bg-accent"
                >
                  <div className="flex h-12 w-12 shrink-0 flex-col items-center justify-center rounded-lg bg-primary/10 text-xs">
                    <span className="font-semibold">{event.startTime.split(':')[0]}</span>
                    <span className="text-muted-foreground">{event.startTime.split(' ')[1]}</span>
                  </div>
                  <div className="flex-1 space-y-1">
                    <h5 className="text-sm font-medium leading-none">{event.title}</h5>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <Clock className="h-3 w-3" />
                      <span>
                        {event.startTime} - {event.endTime}
                      </span>
                    </div>
                    {event.location && (
                      <p className="text-xs text-muted-foreground">{event.location}</p>
                    )}
                    <Badge variant="outline" className={`${getEventColor(event.type)} mt-1 text-xs`}>
                      {event.type}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Upcoming Events */}
        {upcomingEvents.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-semibold">Upcoming</h4>
            <div className="space-y-2">
              {upcomingEvents.map((event) => (
                <div
                  key={event.id}
                  className="flex items-center gap-2 rounded-lg border p-2 text-sm transition-colors hover:bg-accent"
                >
                  <CalendarIcon className="h-4 w-4 text-muted-foreground" />
                  <div className="flex-1">
                    <p className="font-medium">{event.title}</p>
                    <p className="text-xs text-muted-foreground">
                      {formatDate(event.date)} at {event.time}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
