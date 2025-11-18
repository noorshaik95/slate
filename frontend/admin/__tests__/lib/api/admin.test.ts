import axios from 'axios'
import { adminService } from '@/lib/api/admin'

jest.mock('axios')
const mockedAxios = axios as jest.Mocked<typeof axios>

describe('Admin Service', () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  describe('getUsageStats', () => {
    it('should get usage statistics', async () => {
      const mockStats = {
        total_users: 5000,
        active_users: 4200,
        total_storage_used: 209715200, // 200MB
        total_storage_limit: 536870912, // 512MB
        bandwidth_used: 10737418240, // 10GB
        bandwidth_limit: 107374182400, // 100GB
        ai_credits_used: 12000,
        ai_credits_limit: 50000,
        period_start: '2024-01-01',
        period_end: '2024-01-31',
      }

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockStats })

      const result = await adminService.getUsageStats()

      expect(result).toEqual(mockStats)
    })

    it('should get usage stats with date range', async () => {
      mockedAxios.get = jest.fn().mockResolvedValue({ data: {} })

      await adminService.getUsageStats({
        start_date: '2024-01-01',
        end_date: '2024-01-31',
      })

      expect(mockedAxios.get).toHaveBeenCalledWith(
        '/admin/usage/stats',
        expect.objectContaining({
          params: {
            start_date: '2024-01-01',
            end_date: '2024-01-31',
          },
        })
      )
    })
  })

  describe('getBillingInfo', () => {
    it('should get billing information', async () => {
      const mockBilling = {
        current_plan: {
          name: 'Pro',
          price: 299,
          billing_cycle: 'monthly' as const,
          features: ['Feature 1', 'Feature 2'],
        },
        next_billing_date: '2024-02-01',
        payment_method: {
          type: 'card',
          last4: '4242',
        },
        billing_history: [],
      }

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockBilling })

      const result = await adminService.getBillingInfo()

      expect(result).toEqual(mockBilling)
    })
  })

  describe('getPlans', () => {
    it('should get available plans', async () => {
      const mockPlans = [
        {
          id: 'starter',
          name: 'Starter',
          description: 'For small teams',
          price: 99,
          billing_cycle: 'monthly' as const,
          features: ['Feature 1'],
          limits: {
            users: 1000,
            storage: 100,
            bandwidth: 1000,
            ai_credits: 10000,
          },
        },
      ]

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockPlans })

      const result = await adminService.getPlans()

      expect(result).toEqual(mockPlans)
    })
  })

  describe('upgradePlan', () => {
    it('should upgrade to new plan', async () => {
      const mockResponse = {
        success: true,
        message: 'Plan upgraded successfully',
      }

      mockedAxios.post = jest.fn().mockResolvedValue({ data: mockResponse })

      const result = await adminService.upgradePlan('pro')

      expect(result).toEqual(mockResponse)
      expect(mockedAxios.post).toHaveBeenCalledWith('/admin/billing/upgrade', {
        plan_id: 'pro',
      })
    })
  })

  describe('purchaseResources', () => {
    it('should purchase additional resources', async () => {
      const mockPurchase = {
        type: 'storage' as const,
        amount: 100,
        unit_price: 49,
        total_price: 49,
      }

      const mockResponse = {
        success: true,
        message: 'Resources purchased successfully',
      }

      mockedAxios.post = jest.fn().mockResolvedValue({ data: mockResponse })

      const result = await adminService.purchaseResources(mockPurchase)

      expect(result).toEqual(mockResponse)
    })
  })

  describe('getCostOptimization', () => {
    it('should get cost optimization recommendations', async () => {
      const mockOptimization = {
        current_cost: 299,
        potential_savings: 89.7,
        savings_percentage: 30,
        recommendations: [
          {
            id: '1',
            title: 'Optimize Storage',
            description: 'Remove duplicates',
            potential_savings: 22.5,
            impact: 'high' as const,
            implementation_effort: 'easy' as const,
          },
        ],
      }

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockOptimization })

      const result = await adminService.getCostOptimization()

      expect(result).toEqual(mockOptimization)
    })
  })

  describe('applyOptimization', () => {
    it('should apply optimization recommendation', async () => {
      const mockResponse = {
        success: true,
        message: 'Optimization applied successfully',
      }

      mockedAxios.post = jest.fn().mockResolvedValue({ data: mockResponse })

      const result = await adminService.applyOptimization('rec-123')

      expect(result).toEqual(mockResponse)
      expect(mockedAxios.post).toHaveBeenCalledWith('/admin/cost-optimization/rec-123/apply')
    })
  })

  describe('getAnalytics', () => {
    it('should get analytics data', async () => {
      const mockAnalytics = {
        labels: ['Jan', 'Feb', 'Mar'],
        values: [1000, 1200, 1500],
      }

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockAnalytics })

      const result = await adminService.getAnalytics({
        metric: 'users',
        granularity: 'month',
        start_date: '2024-01-01',
        end_date: '2024-03-31',
      })

      expect(result).toEqual(mockAnalytics)
    })
  })
})
