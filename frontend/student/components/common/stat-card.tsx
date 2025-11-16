import * as React from 'react';
import { cn } from '@/lib/utils';
import { GradientType } from './gradient-card';

interface StatCardProps {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  subtitle?: string;
  gradient: GradientType;
  variant?: 'solid' | 'outline';
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

const gradientTextClasses: Record<GradientType, string> = {
  'blue-cyan': 'text-blue-600 dark:text-blue-400',
  'purple-pink': 'text-purple-600 dark:text-purple-400',
  'emerald-teal': 'text-emerald-600 dark:text-emerald-400',
  'orange-red': 'text-orange-600 dark:text-orange-400',
  'amber-yellow': 'text-amber-600 dark:text-amber-400',
  'violet-indigo': 'text-violet-600 dark:text-violet-400',
  'indigo-purple': 'text-indigo-600 dark:text-indigo-400',
};

export const StatCard = React.forwardRef<HTMLDivElement, StatCardProps>(
  ({ icon, label, value, subtitle, gradient, variant = 'outline', className }, ref) => {
    if (variant === 'solid') {
      return (
        <div
          ref={ref}
          className={cn(
            'rounded-2xl p-6',
            gradientClasses[gradient],
            'text-white',
            className
          )}
        >
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <p className="text-sm font-medium text-white/80 mb-1">{label}</p>
              <p className="text-3xl font-bold mb-1">{value}</p>
              {subtitle && <p className="text-sm text-white/70">{subtitle}</p>}
            </div>
            <div className="ml-4 p-3 bg-white/20 backdrop-blur-sm rounded-xl">
              {icon}
            </div>
          </div>
        </div>
      );
    }

    return (
      <div
        ref={ref}
        className={cn(
          'rounded-2xl p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700',
          className
        )}
      >
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <p className="text-sm font-medium text-gray-600 dark:text-gray-400 mb-1">
              {label}
            </p>
            <p className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
              {value}
            </p>
            {subtitle && (
              <p className="text-sm text-gray-500 dark:text-gray-400">{subtitle}</p>
            )}
          </div>
          <div
            className={cn(
              'ml-4 p-3 rounded-xl',
              gradientClasses[gradient],
              'text-white'
            )}
          >
            {icon}
          </div>
        </div>
      </div>
    );
  }
);

StatCard.displayName = 'StatCard';
