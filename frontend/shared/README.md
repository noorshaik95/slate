# @slate/shared

Shared UI components, hooks, and utilities for Slate LMS frontend applications.

## Structure

```
shared/
├── components/ui/       # Reusable UI components
├── hooks/              # Common React hooks
├── utils/              # Utility functions
├── types/              # TypeScript type definitions
└── lib/                # Core libraries (API client, etc.)
```

## Usage

Import from the shared package in any frontend app:

```tsx
import { Button, Card } from '@slate/shared/components/ui';
import { useToast } from '@slate/shared/hooks';
import { cn, formatDate } from '@slate/shared/utils';
```

## Components

All components are built with:
- Radix UI primitives for accessibility
- Tailwind CSS for styling
- TypeScript for type safety
- WCAG 2.1 Level AA compliance

## Principles

1. **Reusability**: Components work across all frontend apps
2. **Accessibility**: WCAG 2.1 Level AA compliant
3. **Type Safety**: Full TypeScript coverage
4. **Performance**: Optimized for bundle size
5. **Consistency**: Unified design system
