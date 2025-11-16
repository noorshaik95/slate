import * as React from 'react';
import { cn } from '@/lib/utils';
import { MapPin, Clock } from 'lucide-react';

interface ScheduleEventProps {
  event: {
    id: string;
    title: string;
    type: 'class' | 'deadline' | 'event';
    time: string;
    endTime?: string;
    location?: string;
    course?: string;
  };
  variant?: 'compact' | 'detailed';
  className?: string;
}

const eventTypeStyles = {
  class: {
    background: 'bg-blue-50 dark:bg-blue-900/20',
    border: 'border-blue-200 dark:border-blue-800',
    badge: 'bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300',
    text: 'text-blue-900 dark:text-blue-100',
  },
  deadline: {
    background: 'bg-red-50 dark:bg-red-900/20',
    border: 'border-red-200 dark:border-red-800',
    badge: 'bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300',
    text: 'text-red-900 dark:text-red-100',
  },
  event: {
    background: 'bg-purple-50 dark:bg-purple-900/20',
    border: 'border-purple-200 dark:border-purple-800',
    badge: 'bg-purple-100 dark:bg-purple-900 text-purple-700 dark:text-purple-300',
    text: 'text-purple-900 dark:text-purple-100',
  },
};

const formatTime = (time: string): { main: string; period: string } => {
  // If time already has AM/PM, parse it
  const match = time.match(/(\d{1,2}):(\d{2})\s*(AM|PM)/i);
  if (match) {
    return {
      main: `${match[1]}:${match[2]}`,
      period: match[3].toUpperCase(),
    };
  }

  // Otherwise, assume it's in 24-hour format
  const [hours, minutes] = time.split(':').map(Number);
  const period = hours >= 12 ? 'PM' : 'AM';
  const displayHours = hours % 12 || 12;
  return {
    main: `${displayHours}:${minutes.toString().padStart(2, '0')}`,
    period,
  };
};

export const ScheduleEvent = React.forwardRef<HTMLDivElement, ScheduleEventProps>(
  ({ event, variant = 'detailed', className }, ref) => {
    const styles = eventTypeStyles[event.type];
    const timeFormatted = formatTime(event.time);

    return (
      <div
        ref={ref}
        className={cn(
          'rounded-xl border p-4 transition-all duration-300',
          styles.background,
          styles.border,
          variant === 'detailed' && 'hover:shadow-md',
          className
        )}
      >
        <div className="flex items-start gap-4">
          {/* Time Display */}
          <div className="flex flex-col items-center min-w-[80px]">
            <span className={cn('text-3xl font-bold', styles.text)}>
              {timeFormatted.main}
            </span>
            <span className={cn('text-sm font-medium', styles.text)}>
              {timeFormatted.period}
            </span>
          </div>

          {/* Event Details */}
          <div className="flex-1">
            <div className="flex items-start justify-between gap-2 mb-2">
              <h4 className={cn('font-semibold text-lg', styles.text)}>
                {event.title}
              </h4>
              <span
                className={cn(
                  'px-2.5 py-1 rounded-md text-xs font-medium capitalize whitespace-nowrap',
                  styles.badge
                )}
              >
                {event.type}
              </span>
            </div>

            {variant === 'detailed' && (
              <div className="space-y-1.5">
                {event.location && (
                  <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
                    <MapPin className="w-4 h-4" />
                    <span>{event.location}</span>
                  </div>
                )}
                {event.course && (
                  <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
                    <Clock className="w-4 h-4" />
                    <span>{event.course}</span>
                  </div>
                )}
                {event.endTime && (
                  <div className="text-sm text-gray-600 dark:text-gray-400">
                    Until {formatTime(event.endTime).main} {formatTime(event.endTime).period}
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  }
);

ScheduleEvent.displayName = 'ScheduleEvent';
