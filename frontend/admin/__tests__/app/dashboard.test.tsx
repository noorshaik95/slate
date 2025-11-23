import { render, screen } from '@testing-library/react'
import DashboardPage from '@/app/dashboard/page'
import { adminService } from '@/lib/api/admin'

jest.mock('@/lib/api/admin')
jest.mock('@/components/layout/dashboard-layout', () => ({
  DashboardLayout: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}))

const mockAdminService = adminService as jest.Mocked<typeof adminService>

describe('DashboardPage', () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  it('should render dashboard title', () => {
    mockAdminService.getUsageStats.mockResolvedValue({
      total_users: 6500,
      active_users: 5500,
      total_storage_used: 209715200,
      total_storage_limit: 536870912,
      bandwidth_used: 10737418240,
      bandwidth_limit: 107374182400,
      ai_credits_used: 14200,
      ai_credits_limit: 50000,
      period_start: '2024-01-01',
      period_end: '2024-01-31',
    })

    render(<DashboardPage />)

    expect(screen.getByText('Dashboard')).toBeInTheDocument()
    expect(screen.getByText(/welcome to the university admin dashboard/i)).toBeInTheDocument()
  })

  it('should display stat cards', () => {
    mockAdminService.getUsageStats.mockResolvedValue({
      total_users: 6500,
      active_users: 5500,
      total_storage_used: 209715200,
      total_storage_limit: 536870912,
      bandwidth_used: 10737418240,
      bandwidth_limit: 107374182400,
      ai_credits_used: 14200,
      ai_credits_limit: 50000,
      period_start: '2024-01-01',
      period_end: '2024-01-31',
    })

    render(<DashboardPage />)

    expect(screen.getByText('Total Users')).toBeInTheDocument()
    expect(screen.getByText('Storage Used')).toBeInTheDocument()
    expect(screen.getByText('Bandwidth')).toBeInTheDocument()
    expect(screen.getByText('AI Credits')).toBeInTheDocument()
  })

  it('should display quick actions', () => {
    mockAdminService.getUsageStats.mockResolvedValue({
      total_users: 0,
      active_users: 0,
      total_storage_used: 0,
      total_storage_limit: 0,
      bandwidth_used: 0,
      bandwidth_limit: 0,
      ai_credits_used: 0,
      ai_credits_limit: 0,
      period_start: '2024-01-01',
      period_end: '2024-01-31',
    })

    render(<DashboardPage />)

    expect(screen.getByText('Quick Actions')).toBeInTheDocument()
    expect(screen.getByText('Bulk Import Users')).toBeInTheDocument()
    expect(screen.getByText('Manage Billing')).toBeInTheDocument()
    expect(screen.getByText('Manage IAM Users')).toBeInTheDocument()
  })

  it('should display system status', () => {
    mockAdminService.getUsageStats.mockResolvedValue({
      total_users: 0,
      active_users: 0,
      total_storage_used: 0,
      total_storage_limit: 0,
      bandwidth_used: 0,
      bandwidth_limit: 0,
      ai_credits_used: 0,
      ai_credits_limit: 0,
      period_start: '2024-01-01',
      period_end: '2024-01-31',
    })

    render(<DashboardPage />)

    expect(screen.getByText('System Status')).toBeInTheDocument()
    expect(screen.getByText('API Gateway')).toBeInTheDocument()
    expect(screen.getByText('Database')).toBeInTheDocument()
    expect(screen.getByText('Storage Service')).toBeInTheDocument()
    expect(screen.getByText('Onboarding Service')).toBeInTheDocument()
  })
})
