# Instructor/Provider Portal

A comprehensive instructor portal for course and student management, built with Next.js 14, React, TypeScript, and Tailwind CSS.

## Features

### Implemented (Story 2.1: Instructor Dashboard)

- **Dashboard Overview** - Welcome screen with key metrics and widgets
- **Course Management Widget** - View and manage all active courses
- **Grading Queue Widget** - Track assignments waiting to be graded
- **Upcoming Classes Widget** - Schedule overview for lectures, labs, and office hours
- **Recent Activity Widget** - Monitor student submissions, questions, and activity
- **Student Performance Widget** - Analytics on grades, engagement, and at-risk students
- **Responsive Design** - Works on desktop, tablet, and mobile devices
- **Dark Mode Support** - Light, dark, and system theme options
- **Mock Data** - Comprehensive mock data for testing and development

## Tech Stack

- **Framework:** Next.js 14 (App Router)
- **Language:** TypeScript
- **Styling:** Tailwind CSS
- **UI Components:** Radix UI primitives
- **Icons:** Lucide React
- **State Management:** React hooks + Zustand (planned)
- **Charts:** Recharts (for analytics)

## Getting Started

### Prerequisites

- Node.js 18+
- npm or yarn

### Installation

```bash
# Install dependencies
npm install

# Create environment file
cp .env.example .env

# Run development server
npm run dev
```

The application will be available at `http://localhost:3001`

### Build for Production

```bash
npm run build
npm run start
```

## Project Structure

```
frontend/provider/
├── app/                        # Next.js app router
│   ├── dashboard/             # Dashboard page
│   ├── courses/               # Courses page (placeholder)
│   ├── grading/               # Grading page (placeholder)
│   ├── analytics/             # Analytics page (placeholder)
│   ├── settings/              # Settings page (placeholder)
│   ├── layout.tsx             # Root layout with sidebar/header
│   ├── page.tsx               # Root redirect to dashboard
│   ├── providers.tsx          # App providers (theme, etc.)
│   └── globals.css            # Global styles
├── components/
│   ├── dashboard/             # Dashboard widgets
│   │   ├── course-overview-widget.tsx
│   │   ├── grading-queue-widget.tsx
│   │   ├── upcoming-classes-widget.tsx
│   │   ├── recent-activity-widget.tsx
│   │   └── student-performance-widget.tsx
│   ├── layout/                # Layout components
│   │   ├── header.tsx
│   │   └── sidebar.tsx
│   ├── providers/             # Provider components
│   │   └── theme-provider.tsx
│   └── ui/                    # Reusable UI components
│       ├── button.tsx
│       ├── card.tsx
│       ├── badge.tsx
│       ├── progress.tsx
│       ├── dropdown-menu.tsx
│       └── tabs.tsx
├── lib/
│   ├── mock-data.ts           # Mock data for development
│   └── utils.ts               # Utility functions
└── types/
    └── index.ts               # TypeScript type definitions
```

## Features Roadmap

### ✅ Completed
- [x] Story 2.1: Instructor Dashboard (21 points)
  - [x] Dashboard grid system
  - [x] Course overview widget
  - [x] Grading queue widget
  - [x] Upcoming classes widget
  - [x] Recent activity widget
  - [x] Student performance widget

### 🚧 In Progress
- [ ] Story 2.2: Course Creation Wizard (34 points)
- [ ] Story 2.3: Grading Interface (34 points)
- [ ] Story 2.4: Content Management (34 points)
- [ ] Story 2.5: Student Analytics Dashboard (34 points)
- [ ] Story 2.6: Communication Center (21 points)

## Dashboard Widgets

### Course Overview Widget
Displays all active courses with:
- Student count
- Pending grading items
- Average grade
- Course progress
- Quick actions

### Grading Queue Widget
Shows assignments waiting to be graded:
- Student information
- Assignment details
- Priority indicators
- Overdue alerts
- Quick grade actions

### Upcoming Classes Widget
Schedule overview including:
- Lectures and labs
- Office hours
- Exams
- Preparation notes
- Location and time details

### Recent Activity Widget
Real-time student activity feed:
- Submissions
- Questions
- Completions
- Discussion posts

### Student Performance Widget
Analytics and insights:
- Average grade across all courses
- Active student count
- Grade distribution
- At-risk student alerts
- Engagement metrics

## Customization

### Theme Configuration
The portal supports light, dark, and system themes. Theme settings are persisted in localStorage.

### Mock Data
Mock data is located in `lib/mock-data.ts`. Modify this file to test different scenarios.

## Contributing

This is part of a larger LMS project. Please follow the project's contribution guidelines.

## License

Proprietary - All rights reserved
