'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Calendar as CalendarIcon, Clock, MapPin, Plus, ChevronLeft, ChevronRight } from 'lucide-react';
import { mockCalendarEvents } from '@/lib/mock-data';

export default function CalendarPage() {
  const [currentDate, setCurrentDate] = useState(new Date());

  const todayEvents = mockCalendarEvents.filter(event => {
    const eventDate = new Date(event.startTime);
    return eventDate.toDateString() === currentDate.toDateString();
  });

  const upcomingEvents = mockCalendarEvents.filter(event => {
    const eventDate = new Date(event.startTime);
    return eventDate > currentDate;
  });

  const getEventColor = (type: string) => {
    switch (type) {
      case 'class': return 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300';
      case 'exam': return 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300';
      case 'deadline': return 'bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300';
      default: return 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Calendar</h1>
          <p className="text-muted-foreground">Manage your schedule and upcoming events</p>
        </div>
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Add Event
        </Button>
      </div>

      <div className="grid gap-6 md:grid-cols-3">
        {/* Calendar View (Simplified) */}
        <Card className="md:col-span-2">
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle>{currentDate.toLocaleDateString('en-US', { month: 'long', year: 'numeric' })}</CardTitle>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  size="icon"
                  onClick={() => setCurrentDate(new Date(currentDate.setMonth(currentDate.getMonth() - 1)))}
                >
                  <ChevronLeft className="h-4 w-4" />
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setCurrentDate(new Date())}
                >
                  Today
                </Button>
                <Button
                  variant="outline"
                  size="icon"
                  onClick={() => setCurrentDate(new Date(currentDate.setMonth(currentDate.getMonth() + 1)))}
                >
                  <ChevronRight className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-7 gap-px bg-gray-200 dark:bg-gray-700 rounded-lg overflow-hidden">
              {['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'].map(day => (
                <div key={day} className="bg-background p-2 text-center text-sm font-medium">
                  {day}
                </div>
              ))}
              {Array.from({ length: 35 }, (_, i) => {
                const day = i - new Date(currentDate.getFullYear(), currentDate.getMonth(), 1).getDay() + 1;
                const isCurrentMonth = day > 0 && day <= new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 0).getDate();
                const isToday = isCurrentMonth && day === new Date().getDate() && currentDate.getMonth() === new Date().getMonth();

                return (
                  <div
                    key={i}
                    className={`bg-background p-2 min-h-[80px] ${!isCurrentMonth && 'text-muted-foreground'} ${isToday && 'ring-2 ring-primary'}`}
                  >
                    {isCurrentMonth && (
                      <span className={`text-sm ${isToday && 'font-bold text-primary'}`}>{day}</span>
                    )}
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>

        {/* Today's Events */}
        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Today's Events</CardTitle>
              <CardDescription>{todayEvents.length} events</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {todayEvents.length > 0 ? (
                todayEvents.map(event => (
                  <div key={event.id} className="space-y-2 pb-3 border-b last:border-0">
                    <div className="flex items-start justify-between">
                      <h4 className="font-medium">{event.title}</h4>
                      <Badge className={getEventColor(event.type)} variant="secondary">
                        {event.type}
                      </Badge>
                    </div>
                    <div className="space-y-1 text-sm text-muted-foreground">
                      <div className="flex items-center gap-2">
                        <Clock className="h-3 w-3" />
                        <span>
                          {new Date(event.startTime).toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' })}
                          {' - '}
                          {new Date(event.endTime).toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' })}
                        </span>
                      </div>
                      {event.location && (
                        <div className="flex items-center gap-2">
                          <MapPin className="h-3 w-3" />
                          <span>{event.location}</span>
                        </div>
                      )}
                    </div>
                  </div>
                ))
              ) : (
                <p className="text-sm text-muted-foreground">No events today</p>
              )}
            </CardContent>
          </Card>

          {/* Upcoming Events */}
          <Card>
            <CardHeader>
              <CardTitle>Upcoming</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {upcomingEvents.slice(0, 5).map(event => (
                <div key={event.id} className="space-y-1">
                  <div className="flex items-center justify-between">
                    <p className="font-medium text-sm">{event.title}</p>
                    <Badge className={getEventColor(event.type)} variant="secondary" className="text-xs">
                      {event.type}
                    </Badge>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {new Date(event.startTime).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
                    {' at '}
                    {new Date(event.startTime).toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' })}
                  </p>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
