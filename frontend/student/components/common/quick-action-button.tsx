import * as React from 'react';
import Link from 'next/link';
import { cn } from '@/lib/utils';
import { GradientType } from './gradient-card';

interface QuickActionButtonProps {
  icon: React.ReactNode;
  label: string;
  gradient: GradientType;
  onClick?: () => void;
  href?: string;
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

export const QuickActionButton = React.forwardRef<
  HTMLButtonElement | HTMLAnchorElement,
  QuickActionButtonProps
>(({ icon, label, gradient, onClick, href, className }, ref) => {
  const content = (
    <>
      <div
        className={cn(
          'w-12 h-12 rounded-xl flex items-center justify-center text-white mb-3',
          gradientClasses[gradient]
        )}
        aria-hidden="true"
      >
        {icon}
      </div>
      <span className="text-sm font-medium text-gray-700 dark:text-gray-300 text-center">
        {label}
      </span>
    </>
  );

  const baseClasses = cn(
    'flex flex-col items-center justify-center p-4 rounded-2xl',
    'bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700',
    'hover-lift cursor-pointer',
    'focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2',
    'transition-all duration-300',
    className
  );

  if (href) {
    return (
      <Link
        href={href}
        className={baseClasses}
        aria-label={label}
        ref={ref as React.Ref<HTMLAnchorElement>}
      >
        {content}
      </Link>
    );
  }

  return (
    <button
      type="button"
      onClick={onClick}
      className={baseClasses}
      aria-label={label}
      ref={ref as React.Ref<HTMLButtonElement>}
    >
      {content}
    </button>
  );
});

QuickActionButton.displayName = 'QuickActionButton';
