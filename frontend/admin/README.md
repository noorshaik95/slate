# University Admin Dashboard

A comprehensive administrative dashboard for managing the university LMS with powerful onboarding, billing, analytics, and IAM capabilities.

## Features

### 🚀 Onboarding Service

- **Bulk Import**: Handle 10,000+ users in under 2 minutes via CSV, API, or directory sync
- **Multiple Integration Methods**: LDAP, SAML, Google Workspace, Microsoft 365
- **Automated Workflows**: Role-based provisioning with different flows for students vs instructors
- **Real-time Progress**: WebSocket-based progress tracking for bulk operations

### 💼 Admin Service

- **Instant Operations**: Plan upgrades, storage purchases, and resource changes in < 3 seconds
- **Self-Service Billing**: Complete billing management without support tickets
- **Usage Analytics**: Real-time monitoring of users, storage, bandwidth, and AI credits
- **Cost Optimization**: AI-powered recommendations to reduce costs by up to 30%

### 🔐 IAM Policy Management

- **Custom Access Control**: Create IAM policy users with granular permissions
- **Service-Level Permissions**: Control access to admin and onboarding services
- **User Management**: Manage IAM users with custom policies and role assignments

## Tech Stack

- **Framework**: Next.js 14 with React 18
- **Styling**: Tailwind CSS with custom design system
- **UI Components**: Radix UI (headless components)
- **Charts**: Recharts for data visualization
- **State Management**: React Hooks + Context
- **Real-time**: Socket.IO client for WebSocket connections
- **HTTP Client**: Axios with interceptors
- **Forms**: CSV parsing with Papa Parse
- **TypeScript**: Full type safety

## Project Structure

```
frontend/admin/
├── app/                          # Next.js app directory
│   ├── dashboard/                # Main dashboard
│   ├── onboarding/               # Onboarding service pages
│   │   ├── bulk-import/          # Bulk user import
│   │   ├── integrations/         # LDAP, SAML, Google, Microsoft
│   │   └── jobs/                 # Import job monitoring
│   ├── admin-service/            # Admin service pages
│   │   ├── analytics/            # Usage analytics
│   │   ├── billing/              # Billing management
│   │   ├── plans/                # Plans and upgrades
│   │   └── optimization/         # Cost optimization
│   ├── iam/                      # IAM management
│   │   ├── users/                # IAM users
│   │   └── policies/             # IAM policies
│   └── login/                    # Authentication
├── components/                   # React components
│   ├── ui/                       # Base UI components
│   ├── layout/                   # Layout components
│   ├── auth/                     # Auth components
│   ├── onboarding/               # Onboarding components
│   ├── admin-service/            # Admin service components
│   └── iam/                      # IAM components
├── lib/                          # Utilities and API clients
│   ├── api/                      # API service clients
│   │   ├── auth.ts               # Authentication
│   │   ├── onboarding.ts         # Onboarding service
│   │   ├── admin.ts              # Admin service
│   │   └── iam.ts                # IAM service
│   ├── utils.ts                  # Utility functions
│   └── websocket.ts              # WebSocket client
└── hooks/                        # Custom React hooks
    └── use-toast.ts              # Toast notifications
```

## Getting Started

### Prerequisites

- Node.js 18+ and npm
- Access to the API Gateway (default: http://localhost:8080)

### Installation

```bash
cd frontend/admin
npm install
```

### Development

```bash
npm run dev
```

The admin dashboard will be available at http://localhost:3001

### Build

```bash
npm run build
npm start
```

## Environment Variables

Create a `.env.local` file in the `frontend/admin` directory:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080
```

## Authentication

The admin dashboard requires users to have the `admin` role. Default credentials for development:

- Email: `admin@example.com`
- Password: `admin123`

**Note**: Change these credentials in production!

## Key Features

### Onboarding Service

#### Bulk Import
- CSV file upload with validation
- API-based bulk import
- Directory sync (LDAP, SAML, Google Workspace, Microsoft 365)
- Real-time progress tracking via WebSocket
- Automatic error handling and reporting
- Support for 10,000+ users in under 2 minutes

#### Integrations
- LDAP/Active Directory integration
- SAML 2.0 SSO
- Google Workspace sync
- Microsoft 365 integration
- One-click synchronization
- Connection testing and validation

#### Job Monitoring
- Real-time job status tracking
- Progress indicators with WebSocket updates
- Detailed error reporting
- Job history and analytics
- Cancel/retry capabilities

### Admin Service

#### Usage Analytics
- Real-time user metrics
- Storage utilization tracking
- Bandwidth monitoring
- AI credit consumption
- Interactive charts and graphs
- Time-range filtering (7d, 30d, 90d, 1y)

#### Billing Management
- Current plan overview
- Payment method management
- Billing history with invoice downloads
- Next billing date and amount
- Automatic renewal settings

#### Plans & Upgrades
- Plan comparison (Starter, Pro, Enterprise)
- One-click plan upgrades
- Add-on purchases (storage, bandwidth, AI credits)
- Feature comparison matrix
- Instant provisioning

#### Cost Optimization
- AI-powered cost analysis
- Savings recommendations
- Implementation difficulty ratings
- Impact assessment
- Category-based optimization (storage, users, AI, bandwidth)
- Potential savings calculation (up to 30%)

### IAM Management

#### IAM Users
- Create and manage policy users
- Assign multiple policies per user
- User status management (active/inactive/suspended)
- Last login tracking
- Search and filter capabilities

#### IAM Policies
- Custom permission creation
- Service-level access control
  - Onboarding Service
  - Admin Service
  - Billing Service
  - Analytics Service
  - IAM Service
- Action-based permissions (create, read, update, delete, etc.)
- User assignment tracking

## API Integration

The dashboard integrates with the following backend services:

- **Authentication Service**: User login, token validation
- **Onboarding Service**: Bulk imports, integrations, job management
- **Admin Service**: Usage stats, billing, cost optimization
- **IAM Service**: Policy and user management

All API calls are made through the centralized API client with:
- Automatic token injection
- 401 handling and redirect to login
- Request/response interceptors
- Error handling

## WebSocket Integration

Real-time updates are handled via Socket.IO for:
- Bulk import progress tracking
- Job status updates
- Usage metrics streaming

## UI Components

The dashboard uses a custom design system built on:
- Radix UI primitives for accessibility
- Tailwind CSS for styling
- Custom animations and transitions
- Dark mode support (via next-themes)
- Responsive design (mobile-first)

## Performance

- Next.js 14 with App Router for optimal performance
- Server-side rendering (SSR) where appropriate
- Client-side caching with React Query
- Code splitting and lazy loading
- Optimized bundle size
- Fast page transitions

## Security

- JWT-based authentication
- Role-based access control (RBAC)
- Protected routes with middleware
- Secure API communication
- XSS and CSRF protection
- Security headers configured

## Testing

The admin dashboard includes comprehensive unit and integration tests.

### Running Tests

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage
```

### Test Coverage

The project maintains 70%+ coverage across:
- Utility functions
- API services (auth, onboarding, admin, IAM)
- React hooks
- UI components (button, card, input, badge, etc.)
- Authentication flows
- Page components
- Integration tests

### Test Structure

```
__tests__/
├── lib/                    # Utility and API service tests
├── hooks/                  # Custom hooks tests
├── components/             # Component tests
├── app/                    # Page tests
└── integration/            # Integration tests
```

For detailed testing documentation, see [TESTING.md](./TESTING.md).

## Contributing

1. Follow the existing code structure
2. Use TypeScript for type safety
3. Follow the component naming conventions
4. Add proper error handling
5. Include proper loading states
6. Test responsiveness on all screen sizes

## License

Proprietary - University LMS Admin Dashboard
