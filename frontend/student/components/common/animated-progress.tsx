import * as React from 'react';
import { cn } from '@/lib/utils';
import { GradientType } from './gradient-card';

interface AnimatedProgressProps {
  value: number;
  gradient: GradientType;
  showLabel?: boolean;
  size?: 'sm' | 'md' | 'lg';
  animated?: boolean;
  className?: string;
}

const gradientClasses: Record<GradientType, string> = {
  'blue-cyan': 'gradient-blue-cyan',
  'purple-pink': 'gradient-purple-pink',
  'emerald-teal': 'gradient-emerald-teal',
  'orange-red': 'gradient-orange-red',
  'amber-yellow': 'gradient-amber-yellow',
  'violet-indigo': 'gradient-violet-indigo',
  'indigo-purple': 'gradient-indigo-purple',
};

const sizeClasses = {
  sm: 'h-1.5',
  md: 'h-2.5',
  lg: 'h-3',
};

export const AnimatedProgress = React.forwardRef<HTMLDivElement, AnimatedProgressProps>(
  ({ value, gradient, showLabel = false, size = 'md', animated = true, className }, ref) => {
    const clampedValue = Math.min(Math.max(value, 0), 100);

    return (
      <div ref={ref} className={cn('w-full', className)}>
        <div className="flex items-center justify-between mb-1">
          {showLabel && (
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
              {clampedValue}%
            </span>
          )}
        </div>
        <div
          className={cn(
            'w-full bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden',
            sizeClasses[size]
          )}
        >
          <div
            className={cn(
              'h-full rounded-full transition-all duration-500 ease-out',
              gradientClasses[gradient],
              animated && 'shimmer-animation'
            )}
            style={{ width: `${clampedValue}%` }}
            role="progressbar"
            aria-valuenow={clampedValue}
            aria-valuemin={0}
            aria-valuemax={100}
          />
        </div>
      </div>
    );
  }
);

AnimatedProgress.displayName = 'AnimatedProgress';
