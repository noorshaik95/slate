'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { ChevronRight, Home } from 'lucide-react';
import { Fragment } from 'react';

export function Breadcrumbs() {
  const pathname = usePathname();

  const pathSegments = pathname.split('/').filter(Boolean);

  // Convert path segment to readable label
  const formatLabel = (segment: string) => {
    return segment
      .split('-')
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  };

  return (
    <nav aria-label="Breadcrumb" className="flex items-center space-x-2 text-sm">
      <Link
        href="/dashboard"
        className="flex items-center text-muted-foreground hover:text-foreground focus-ring rounded"
        aria-label="Home"
      >
        <Home className="h-4 w-4" />
      </Link>

      {pathSegments.map((segment, index) => {
        const href = `/${pathSegments.slice(0, index + 1).join('/')}`;
        const isLast = index === pathSegments.length - 1;

        return (
          <Fragment key={segment}>
            <ChevronRight className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
            {isLast ? (
              <span className="font-medium text-foreground" aria-current="page">
                {formatLabel(segment)}
              </span>
            ) : (
              <Link
                href={href}
                className="text-muted-foreground hover:text-foreground focus-ring rounded"
              >
                {formatLabel(segment)}
              </Link>
            )}
          </Fragment>
        );
      })}
    </nav>
  );
}
