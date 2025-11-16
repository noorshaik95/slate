# Student/Instructor Portal - Frontend

A modern, accessible Student/Instructor Portal built with Next.js 14, featuring a comprehensive dashboard, course management, assignments, gradebook, and real-time updates.

## Project Status

### ✅ Completed Features

1. **Project Setup**
   - Next.js 14 with App Router and React Server Components
   - TypeScript configuration with strict mode
   - Tailwind CSS with custom design system
   - ESLint and Prettier for code quality

2. **Core Architecture**
   - State management with Zustand + TanStack Query
   - Real-time updates via Socket.io
   - OpenTelemetry monitoring integration
   - Theme system with dark mode support

3. **Dashboard Layout**
   - Responsive sidebar navigation with collapse functionality
   - Header with search and notifications
   - Breadcrumb navigation
   - Keyboard navigation support (Alt+S for sidebar)
   - Focus management and accessibility features

4. **Student Dashboard Widgets**
   - **Quick Actions Panel**: Fast access to common tasks (submit assignment, join discussion, etc.)
   - **Course Progress Widget**: Displays enrolled courses with progress bars and upcoming deadlines
   - **Upcoming Assignments Widget**: Shows assignments due in next 7 days with countdown timers
   - **Recent Grades Widget**: Latest graded work with trend indicators and letter grades
   - **Announcements Widget**: Course announcements with priority levels and read/unread states
   - **Calendar Widget**: Today's schedule and upcoming events

5. **UI Components (Radix UI + Tailwind)**
   - Button, Card, Input, Badge, Progress, Toast/Toaster
   - All components are accessible (WCAG 2.1 Level AA)
   - Dark mode support built-in

6. **Accessibility Features**
   - Keyboard navigation throughout
   - ARIA labels and semantic HTML
   - Skip to main content link
   - Focus indicators (4.5:1 contrast ratio)
   - Screen reader compatible
   - Reduced motion support

### 📋 Todo: Additional Features to Implement

#### Phase 1: Core Pages (High Priority)

1. **Authentication Pages** (`app/(auth)/`)
   - Login page with email/password
   - Registration for students/instructors
   - Password reset flow
   - OAuth integration (Google, Microsoft)

2. **Course Pages** (`app/courses/`)
   - Course list with filters/search
   - Course detail page with tabs (Content, Assignments, Discussions, Grades, People)
   - Module-based content organization
   - Content type icons (video, document, quiz)
   - Progress tracking per module
   - Download for offline viewing

3. **Assignment Pages** (`app/assignments/`)
   - Assignment list with filters
   - Assignment detail/submission page
   - File upload with drag-and-drop
   - Rich text editor for text submissions
   - Auto-save drafts
   - Submission history
   - Preview before submit
   - Rubric display

4. **Gradebook Page** (`app/grades/`)
   - Overall grade display
   - Grade breakdown by category
   - Detailed feedback viewer
   - Grade trends/analytics charts
   - What-if grade calculator
   - Export grades (PDF/CSV)
   - Class statistics

#### Phase 2: Advanced Features

5. **Discussion Forums** (`app/discussions/`)
   - Thread list with sorting/filtering
   - Thread detail with replies
   - Rich text editor
   - Like/upvote system
   - Search within discussions

6. **Calendar** (`app/calendar/`)
   - Full month/week/day views
   - Event creation/editing
   - Assignment deadlines integration
   - Color coding by course
   - Export to iCal

7. **Profile & Settings** (`app/profile/`, `app/settings/`)
   - Profile editing (avatar, bio)
   - Notification preferences
   - Privacy settings
   - Accessibility settings
   - Theme preferences

8. **Mobile Optimizations**
   - Mobile drawer navigation with gestures
   - Touch-optimized UI
   - Offline mode with service workers
   - PWA installation
   - Push notifications

9. **Performance Features**
   - Code splitting and lazy loading
   - Image optimization
   - Bundle size optimization (< 200KB JS)
   - Caching strategies
   - Core Web Vitals monitoring

## Tech Stack

- **Framework**: Next.js 14.2+ (App Router, RSC)
- **UI Library**: React 18.3+
- **Language**: TypeScript 5.6+
- **Styling**: Tailwind CSS 3.4+
- **Components**: Radix UI primitives
- **State**: Zustand + TanStack Query
- **Real-time**: Socket.io Client
- **Monitoring**: OpenTelemetry
- **Icons**: Lucide React

## Quick Start

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Environment

Create `.env.local`:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080
NEXT_PUBLIC_OTEL_ENDPOINT=http://localhost:4318/v1/traces
NEXT_PUBLIC_OTEL_SERVICE_NAME=student-portal-web
```

### 3. Run Development Server

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000)

### 4. Build for Production

```bash
npm run build
npm start
```

## Project Structure

```
frontend/
├── app/                     # Next.js 14 App Router
│   ├── (auth)/             # Auth pages (login, register)
│   ├── dashboard/          # Dashboard with widgets
│   ├── courses/            # Course pages
│   ├── assignments/        # Assignment pages
│   ├── grades/             # Gradebook
│   ├── discussions/        # Discussion forums
│   ├── calendar/           # Calendar view
│   ├── profile/            # User profile
│   ├── settings/           # User settings
│   ├── layout.tsx          # Root layout
│   ├── page.tsx            # Home page
│   ├── providers.tsx       # React context providers
│   └── globals.css         # Global styles
├── components/
│   ├── ui/                 # Reusable UI components
│   ├── layout/             # Layout components (Sidebar, Header, Breadcrumbs)
│   ├── dashboard/          # Dashboard widgets
│   ├── courses/            # Course components
│   ├── assignments/        # Assignment components
│   ├── grades/             # Grade components
│   └── providers/          # Context providers
├── lib/
│   ├── api/                # API client
│   ├── telemetry/          # OpenTelemetry setup
│   ├── utils.ts            # Utility functions
│   └── validations/        # Zod schemas
├── hooks/                  # Custom React hooks
├── stores/                 # Zustand stores
├── types/                  # TypeScript types
└── public/                 # Static assets

```

## Key Files Created

### Configuration
- `package.json` - Dependencies and scripts
- `tsconfig.json` - TypeScript configuration
- `next.config.js` - Next.js configuration with security headers
- `tailwind.config.ts` - Tailwind with custom theme
- `.eslintrc.json` - ESLint rules
- `.prettierrc` - Code formatting rules

### Core Application
- `app/layout.tsx` - Root layout with providers
- `app/providers.tsx` - React Query, Theme, Socket, Telemetry providers
- `app/globals.css` - Global styles with CSS variables
- `app/page.tsx` - Landing page
- `app/dashboard/layout.tsx` - Dashboard layout
- `app/dashboard/page.tsx` - Dashboard with widgets

### Layout Components
- `components/layout/sidebar.tsx` - Collapsible sidebar navigation
- `components/layout/header.tsx` - Header with search and notifications
- `components/layout/breadcrumbs.tsx` - Dynamic breadcrumb navigation
- `components/layout/theme-toggle.tsx` - Dark mode toggle

### Dashboard Widgets
- `components/dashboard/quick-actions-widget.tsx` - Quick action buttons
- `components/dashboard/course-progress-widget.tsx` - Course progress cards
- `components/dashboard/upcoming-assignments-widget.tsx` - Assignment list with countdown
- `components/dashboard/recent-grades-widget.tsx` - Recent grades with trends
- `components/dashboard/announcements-widget.tsx` - Announcement feed
- `components/dashboard/calendar-widget.tsx` - Today's schedule

### UI Components
- `components/ui/button.tsx` - Button with variants
- `components/ui/card.tsx` - Card component
- `components/ui/input.tsx` - Input field
- `components/ui/badge.tsx` - Badge/tag component
- `components/ui/progress.tsx` - Progress bar
- `components/ui/toast.tsx` - Toast notifications
- `components/ui/toaster.tsx` - Toast container

### Utilities & Types
- `lib/utils.ts` - Helper functions (cn, formatDate, etc.)
- `lib/api/client.ts` - Axios API client with interceptors
- `lib/telemetry/init.ts` - OpenTelemetry initialization
- `types/index.ts` - TypeScript type definitions
- `hooks/use-toast.ts` - Toast notification hook

### Providers
- `components/providers/theme-provider.tsx` - Theme context
- `components/providers/toast-provider.tsx` - Toast notifications
- `components/providers/socket-provider.tsx` - WebSocket connection
- `components/providers/telemetry-provider.tsx` - OpenTelemetry

## Performance Targets

### Core Web Vitals
- **LCP** (Largest Contentful Paint): < 2.5s
- **FID** (First Input Delay): < 100ms
- **CLS** (Cumulative Layout Shift): < 0.1

### Bundle Sizes
- JavaScript: < 200KB gzipped
- CSS: < 50KB gzipped
- Total Page Weight: < 1MB

### Lighthouse Scores (Target)
- Performance: > 90
- Accessibility: > 95
- Best Practices: > 95
- SEO: > 95

## Accessibility Compliance

- WCAG 2.1 Level AA compliant
- Keyboard navigation support
- Screen reader compatible
- 4.5:1 color contrast ratio
- Focus indicators on all interactive elements
- Semantic HTML throughout
- ARIA labels where needed

## Development Workflow

1. **Start development server**: `npm run dev`
2. **Lint code**: `npm run lint`
3. **Type check**: `npm run type-check` (add this script to package.json)
4. **Build**: `npm run build`
5. **Start production**: `npm start`

## Docker Deployment

```bash
# Build image
docker build -t student-portal-frontend .

# Run container
docker run -p 3000:3000 student-portal-frontend
```

## Next Steps

1. Install dependencies: `npm install`
2. Create `.env.local` with environment variables
3. Implement authentication pages
4. Build course pages with modules
5. Create assignment submission system
6. Implement gradebook with analytics
7. Add discussion forums
8. Build calendar interface
9. Implement PWA features
10. Add comprehensive testing

## Support & Documentation

- Architecture decisions are documented in code comments
- All components include TypeScript types
- Accessibility features are marked with ARIA attributes
- Performance optimizations are noted in relevant files

---

**Note**: This is a work in progress. The core dashboard and layout are complete. Additional pages and features need to be implemented as outlined in the Todo section above.
