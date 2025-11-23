# Testing Guide

This document outlines the testing strategy and setup for the Admin Dashboard.

## Testing Stack

- **Test Runner**: Jest 29.7
- **Testing Library**: React Testing Library 14.1
- **User Interactions**: @testing-library/user-event
- **Assertions**: @testing-library/jest-dom
- **Environment**: jsdom (for browser environment simulation)

## Running Tests

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage
```

## Test Structure

```
__tests__/
├── lib/                          # Utility and API service tests
│   ├── utils.test.ts             # Utility function tests
│   └── api/                      # API service tests
│       ├── auth.test.ts          # Authentication service
│       ├── onboarding.test.ts    # Onboarding service
│       ├── admin.test.ts         # Admin service
│       └── iam.test.ts           # IAM service
├── hooks/                        # Custom hooks tests
│   └── use-toast.test.tsx        # Toast hook tests
├── components/                   # Component tests
│   ├── ui/                       # UI component tests
│   │   ├── button.test.tsx
│   │   ├── card.test.tsx
│   │   ├── input.test.tsx
│   │   └── badge.test.tsx
│   └── auth/                     # Auth component tests
│       └── protected-route.test.tsx
├── app/                          # Page component tests
│   ├── login.test.tsx            # Login page
│   └── dashboard.test.tsx        # Dashboard page
└── integration/                  # Integration tests
    └── onboarding-flow.test.tsx  # Full onboarding flow
```

## Coverage Thresholds

The project maintains the following minimum coverage thresholds:

- **Branches**: 70%
- **Functions**: 70%
- **Lines**: 70%
- **Statements**: 70%

## Test Categories

### 1. Unit Tests

**Utility Functions** (`__tests__/lib/utils.test.ts`)
- String formatting (formatBytes, formatCurrency, formatPercentage)
- Number formatting
- Class name utilities
- Helper functions (truncate, debounce, getInitials)

**API Services** (`__tests__/lib/api/*.test.ts`)
- Authentication service (login, logout, token validation)
- Onboarding service (bulk import, integrations, jobs)
- Admin service (usage stats, billing, cost optimization)
- IAM service (policies, users, permissions)

**Hooks** (`__tests__/hooks/*.test.tsx`)
- Toast notifications
- State management
- Side effects

### 2. Component Tests

**UI Components** (`__tests__/components/ui/*.test.tsx`)
- Button interactions and variants
- Card composition
- Input handling
- Badge variants
- Form elements

**Layout Components** (`__tests__/components/layout/*.test.tsx`)
- Dashboard layout
- Navigation
- Protected routes

**Auth Components** (`__tests__/components/auth/*.test.tsx`)
- Protected route logic
- Role-based access control
- Authentication state

### 3. Integration Tests

**Page Tests** (`__tests__/app/*.test.tsx`)
- Login flow
- Dashboard rendering
- Data fetching

**Feature Flows** (`__tests__/integration/*.test.tsx`)
- Complete onboarding workflow
- Multi-step processes
- User journeys

## Writing Tests

### Component Test Example

```typescript
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Button } from '@/components/ui/button'

describe('Button', () => {
  it('should handle click events', async () => {
    const user = userEvent.setup()
    const handleClick = jest.fn()

    render(<Button onClick={handleClick}>Click me</Button>)
    await user.click(screen.getByRole('button'))

    expect(handleClick).toHaveBeenCalledTimes(1)
  })
})
```

### API Service Test Example

```typescript
import { authService } from '@/lib/api/auth'
import axios from 'axios'

jest.mock('axios')
const mockedAxios = axios as jest.Mocked<typeof axios>

describe('Auth Service', () => {
  it('should login successfully', async () => {
    mockedAxios.post.mockResolvedValue({
      data: {
        access_token: 'token',
        user: { id: '1', email: 'admin@test.com', roles: ['admin'] },
      },
    })

    const result = await authService.login({
      email: 'admin@test.com',
      password: 'password',
    })

    expect(result.access_token).toBe('token')
  })
})
```

### Integration Test Example

```typescript
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import LoginPage from '@/app/login/page'

describe('Login Flow', () => {
  it('should complete full login flow', async () => {
    const user = userEvent.setup()

    render(<LoginPage />)

    await user.type(screen.getByLabelText(/email/i), 'admin@test.com')
    await user.type(screen.getByLabelText(/password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /sign in/i }))

    await waitFor(() => {
      expect(mockRouter.push).toHaveBeenCalledWith('/dashboard')
    })
  })
})
```

## Mocking

### Next.js Router

The router is automatically mocked in `jest.setup.js`:

```javascript
jest.mock('next/navigation', () => ({
  useRouter() {
    return {
      push: jest.fn(),
      replace: jest.fn(),
      pathname: '/',
    }
  },
}))
```

### localStorage

localStorage is mocked globally in `jest.setup.js`:

```javascript
const localStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
}
global.localStorage = localStorageMock
```

### API Clients

API services should be mocked per test file:

```typescript
jest.mock('@/lib/api/auth')
const mockAuthService = authService as jest.Mocked<typeof authService>
```

## Best Practices

1. **Arrange-Act-Assert**: Structure tests clearly
   ```typescript
   // Arrange
   const mockData = { ... }

   // Act
   const result = performAction(mockData)

   // Assert
   expect(result).toBe(expected)
   ```

2. **User-Centric Testing**: Test from the user's perspective
   ```typescript
   // Good
   await user.click(screen.getByRole('button', { name: /submit/i }))

   // Avoid
   fireEvent.click(document.querySelector('.submit-btn'))
   ```

3. **Async Handling**: Use waitFor for async operations
   ```typescript
   await waitFor(() => {
     expect(screen.getByText('Success')).toBeInTheDocument()
   })
   ```

4. **Cleanup**: Jest automatically cleans up after each test, but clear mocks:
   ```typescript
   beforeEach(() => {
     jest.clearAllMocks()
   })
   ```

5. **Descriptive Test Names**: Use clear, descriptive test names
   ```typescript
   it('should redirect to login if user is not admin', () => {
     // test implementation
   })
   ```

## Common Patterns

### Testing Forms

```typescript
const user = userEvent.setup()
await user.type(screen.getByLabelText(/email/i), 'test@example.com')
await user.click(screen.getByRole('button', { name: /submit/i }))
```

### Testing API Calls

```typescript
mockedAxios.get.mockResolvedValue({ data: mockData })
const result = await service.getData()
expect(result).toEqual(mockData)
```

### Testing Loading States

```typescript
render(<Component />)
expect(screen.getByRole('progressbar')).toBeInTheDocument()

await waitFor(() => {
  expect(screen.queryByRole('progressbar')).not.toBeInTheDocument()
})
```

### Testing Error States

```typescript
mockedService.fetch.mockRejectedValue(new Error('Network error'))
render(<Component />)

await waitFor(() => {
  expect(screen.getByText(/error/i)).toBeInTheDocument()
})
```

## Debugging Tests

### Run Specific Test

```bash
npm test -- button.test.tsx
```

### Run Tests Matching Pattern

```bash
npm test -- --testNamePattern="should handle click"
```

### Debug Mode

```bash
node --inspect-brk node_modules/.bin/jest --runInBand
```

### View Test Output

```bash
npm test -- --verbose
```

## Continuous Integration

Tests run automatically on:
- Pre-commit (via git hooks)
- Pull requests
- Main branch merges

CI fails if:
- Any test fails
- Coverage drops below thresholds
- Linting errors exist

## Resources

- [Jest Documentation](https://jestjs.io/)
- [React Testing Library](https://testing-library.com/react)
- [Testing Best Practices](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)
- [User Event Documentation](https://testing-library.com/docs/user-event/intro)
