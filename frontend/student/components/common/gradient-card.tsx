import * as React from 'react';
import { cn } from '@/lib/utils';

export type GradientType =
  | 'blue-cyan'
  | 'purple-pink'
  | 'emerald-teal'
  | 'orange-red'
  | 'amber-yellow'
  | 'violet-indigo'
  | 'indigo-purple';

interface GradientCardProps extends React.HTMLAttributes<HTMLDivElement> {
  gradient: GradientType;
  children: React.ReactNode;
  glassEffect?: boolean;
  hoverLift?: boolean;
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

export const GradientCard = React.forwardRef<HTMLDivElement, GradientCardProps>(
  ({ gradient, children, className, glassEffect = false, hoverLift = false, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          'rounded-2xl p-6',
          gradientClasses[gradient],
          glassEffect && 'glass-effect',
          hoverLift && 'hover-lift',
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);

GradientCard.displayName = 'GradientCard';
