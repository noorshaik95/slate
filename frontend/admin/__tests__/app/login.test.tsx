import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import LoginPage from '@/app/login/page'
import { authService } from '@/lib/api/auth'
import { useRouter } from 'next/navigation'
import { toast } from '@/hooks/use-toast'

jest.mock('@/lib/api/auth')
jest.mock('next/navigation')
jest.mock('@/hooks/use-toast')

const mockAuthService = authService as jest.Mocked<typeof authService>
const mockUseRouter = useRouter as jest.MockedFunction<typeof useRouter>
const mockToast = toast as jest.MockedFunction<typeof toast>

describe('LoginPage', () => {
  const mockPush = jest.fn()

  beforeEach(() => {
    jest.clearAllMocks()
    mockUseRouter.mockReturnValue({
      push: mockPush,
      replace: jest.fn(),
      prefetch: jest.fn(),
      back: jest.fn(),
      pathname: '/login',
      query: {},
      asPath: '/login',
    } as any)
    mockToast.mockReturnValue({
      id: '1',
      dismiss: jest.fn(),
      update: jest.fn(),
    })
  })

  it('should render login form', () => {
    render(<LoginPage />)

    expect(screen.getByLabelText(/email/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument()
  })

  it('should handle successful admin login', async () => {
    const user = userEvent.setup()

    mockAuthService.login.mockResolvedValue({
      access_token: 'test-token',
      refresh_token: 'refresh-token',
      user: {
        id: '1',
        email: 'admin@test.com',
        roles: ['admin'],
      },
    })

    render(<LoginPage />)

    await user.type(screen.getByLabelText(/email/i), 'admin@test.com')
    await user.type(screen.getByLabelText(/password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /sign in/i }))

    await waitFor(() => {
      expect(mockAuthService.login).toHaveBeenCalledWith({
        email: 'admin@test.com',
        password: 'password123',
      })
      expect(mockPush).toHaveBeenCalledWith('/dashboard')
    })
  })

  it('should reject non-admin users', async () => {
    const user = userEvent.setup()

    mockAuthService.login.mockResolvedValue({
      access_token: 'test-token',
      refresh_token: 'refresh-token',
      user: {
        id: '1',
        email: 'user@test.com',
        roles: ['user'],
      },
    })

    mockAuthService.logout.mockResolvedValue()

    render(<LoginPage />)

    await user.type(screen.getByLabelText(/email/i), 'user@test.com')
    await user.type(screen.getByLabelText(/password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /sign in/i }))

    await waitFor(() => {
      expect(mockAuthService.logout).toHaveBeenCalled()
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Access Denied',
          variant: 'destructive',
        })
      )
      expect(mockPush).not.toHaveBeenCalled()
    })
  })

  it('should handle login failure', async () => {
    const user = userEvent.setup()

    mockAuthService.login.mockRejectedValue({
      response: {
        data: {
          message: 'Invalid credentials',
        },
      },
    })

    render(<LoginPage />)

    await user.type(screen.getByLabelText(/email/i), 'wrong@test.com')
    await user.type(screen.getByLabelText(/password/i), 'wrongpassword')
    await user.click(screen.getByRole('button', { name: /sign in/i }))

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Login Failed',
          variant: 'destructive',
        })
      )
    })
  })

  it('should disable form while submitting', async () => {
    const user = userEvent.setup()

    mockAuthService.login.mockImplementation(
      () => new Promise((resolve) => setTimeout(resolve, 1000))
    )

    render(<LoginPage />)

    await user.type(screen.getByLabelText(/email/i), 'admin@test.com')
    await user.type(screen.getByLabelText(/password/i), 'password123')

    const submitButton = screen.getByRole('button', { name: /sign in/i })
    await user.click(submitButton)

    expect(submitButton).toHaveTextContent(/signing in/i)
    expect(screen.getByLabelText(/email/i)).toBeDisabled()
    expect(screen.getByLabelText(/password/i)).toBeDisabled()
  })
})
