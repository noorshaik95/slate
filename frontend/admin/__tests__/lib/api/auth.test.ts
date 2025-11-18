import axios from 'axios'
import { authService } from '@/lib/api/auth'

jest.mock('axios')
const mockedAxios = axios as jest.Mocked<typeof axios>

describe('Auth Service', () => {
  beforeEach(() => {
    jest.clearAllMocks()
    localStorage.clear()
  })

  describe('login', () => {
    it('should login successfully and store token', async () => {
      const mockResponse = {
        data: {
          access_token: 'test-token',
          refresh_token: 'refresh-token',
          user: {
            id: '1',
            email: 'admin@test.com',
            roles: ['admin'],
          },
        },
      }

      mockedAxios.post = jest.fn().mockResolvedValue(mockResponse)

      const result = await authService.login({
        email: 'admin@test.com',
        password: 'password123',
      })

      expect(result).toEqual(mockResponse.data)
      expect(localStorage.setItem).toHaveBeenCalledWith('admin_auth_token', 'test-token')
    })

    it('should handle login failure', async () => {
      mockedAxios.post = jest.fn().mockRejectedValue(new Error('Invalid credentials'))

      await expect(
        authService.login({
          email: 'wrong@test.com',
          password: 'wrongpass',
        })
      ).rejects.toThrow('Invalid credentials')
    })
  })

  describe('logout', () => {
    it('should logout and remove token', async () => {
      mockedAxios.post = jest.fn().mockResolvedValue({})
      localStorage.setItem('admin_auth_token', 'test-token')

      await authService.logout()

      expect(localStorage.removeItem).toHaveBeenCalledWith('admin_auth_token')
    })

    it('should remove token even if API call fails', async () => {
      mockedAxios.post = jest.fn().mockRejectedValue(new Error('Network error'))
      localStorage.setItem('admin_auth_token', 'test-token')

      await authService.logout()

      expect(localStorage.removeItem).toHaveBeenCalledWith('admin_auth_token')
    })
  })

  describe('getProfile', () => {
    it('should get user profile', async () => {
      const mockProfile = {
        id: '1',
        email: 'admin@test.com',
        name: 'Admin User',
        roles: ['admin'],
        created_at: '2024-01-01',
      }

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockProfile })

      const result = await authService.getProfile()

      expect(result).toEqual(mockProfile)
    })
  })

  describe('validateToken', () => {
    it('should return true for valid token', async () => {
      mockedAxios.post = jest.fn().mockResolvedValue({ data: { valid: true } })

      const result = await authService.validateToken()

      expect(result).toBe(true)
    })

    it('should return false for invalid token', async () => {
      mockedAxios.post = jest.fn().mockRejectedValue(new Error('Invalid token'))

      const result = await authService.validateToken()

      expect(result).toBe(false)
    })
  })

  describe('getToken', () => {
    it('should get token from localStorage', () => {
      localStorage.setItem('admin_auth_token', 'test-token')

      const token = authService.getToken()

      expect(token).toBe('test-token')
    })

    it('should return null if no token', () => {
      const token = authService.getToken()

      expect(token).toBe(null)
    })
  })

  describe('isAuthenticated', () => {
    it('should return true if token exists', () => {
      localStorage.setItem('admin_auth_token', 'test-token')

      expect(authService.isAuthenticated()).toBe(true)
    })

    it('should return false if no token', () => {
      expect(authService.isAuthenticated()).toBe(false)
    })
  })
})
