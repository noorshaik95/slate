import { render, screen, waitFor } from '@testing-library/react'
import { ProtectedRoute } from '@/components/auth/protected-route'
import { authService } from '@/lib/api/auth'
import { useRouter } from 'next/navigation'

jest.mock('@/lib/api/auth')
jest.mock('next/navigation')

const mockAuthService = authService as jest.Mocked<typeof authService>
const mockUseRouter = useRouter as jest.MockedFunction<typeof useRouter>

describe('ProtectedRoute', () => {
  const mockPush = jest.fn()

  beforeEach(() => {
    jest.clearAllMocks()
    mockUseRouter.mockReturnValue({
      push: mockPush,
      replace: jest.fn(),
      prefetch: jest.fn(),
      back: jest.fn(),
      pathname: '/',
      query: {},
      asPath: '/',
    } as any)
  })

  it('should show loading spinner while checking auth', () => {
    mockAuthService.getToken.mockReturnValue('test-token')
    mockAuthService.getProfile.mockReturnValue(
      new Promise(() => {}) // Never resolves to keep loading
    )

    render(
      <ProtectedRoute>
        <div>Protected Content</div>
      </ProtectedRoute>
    )

    expect(screen.getByRole('progressbar', { hidden: true })).toBeInTheDocument()
  })

  it('should redirect to login if no token', async () => {
    mockAuthService.getToken.mockReturnValue(null)

    render(
      <ProtectedRoute>
        <div>Protected Content</div>
      </ProtectedRoute>
    )

    await waitFor(() => {
      expect(mockPush).toHaveBeenCalledWith('/login')
    })
  })

  it('should redirect to login if user is not admin', async () => {
    mockAuthService.getToken.mockReturnValue('test-token')
    mockAuthService.getProfile.mockResolvedValue({
      id: '1',
      email: 'user@test.com',
      name: 'Regular User',
      roles: ['user'],
      created_at: '2024-01-01',
    })

    render(
      <ProtectedRoute>
        <div>Protected Content</div>
      </ProtectedRoute>
    )

    await waitFor(() => {
      expect(mockPush).toHaveBeenCalledWith('/login')
    })
  })

  it('should render children if user is admin', async () => {
    mockAuthService.getToken.mockReturnValue('test-token')
    mockAuthService.getProfile.mockResolvedValue({
      id: '1',
      email: 'admin@test.com',
      name: 'Admin User',
      roles: ['admin'],
      created_at: '2024-01-01',
    })

    render(
      <ProtectedRoute>
        <div>Protected Content</div>
      </ProtectedRoute>
    )

    await waitFor(() => {
      expect(screen.getByText('Protected Content')).toBeInTheDocument()
    })
  })

  it('should redirect to login if API call fails', async () => {
    mockAuthService.getToken.mockReturnValue('test-token')
    mockAuthService.getProfile.mockRejectedValue(new Error('Unauthorized'))

    render(
      <ProtectedRoute>
        <div>Protected Content</div>
      </ProtectedRoute>
    )

    await waitFor(() => {
      expect(mockPush).toHaveBeenCalledWith('/login')
    })
  })
})
