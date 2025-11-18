import axios from 'axios'
import { iamService } from '@/lib/api/iam'

jest.mock('axios')
const mockedAxios = axios as jest.Mocked<typeof axios>

describe('IAM Service', () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  describe('listPolicies', () => {
    it('should list all policies', async () => {
      const mockPolicies = [
        {
          id: '1',
          name: 'Full Access',
          description: 'Complete access',
          permissions: [
            {
              service: 'onboarding' as const,
              actions: ['*'],
            },
          ],
          created_at: '2024-01-01',
          updated_at: '2024-01-01',
        },
      ]

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockPolicies })

      const result = await iamService.listPolicies()

      expect(result).toEqual(mockPolicies)
    })
  })

  describe('getPolicy', () => {
    it('should get a specific policy', async () => {
      const mockPolicy = {
        id: '1',
        name: 'Onboarding Admin',
        description: 'Onboarding access',
        permissions: [
          {
            service: 'onboarding' as const,
            actions: ['create', 'read', 'update'],
          },
        ],
        created_at: '2024-01-01',
        updated_at: '2024-01-01',
      }

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockPolicy })

      const result = await iamService.getPolicy('1')

      expect(result).toEqual(mockPolicy)
    })
  })

  describe('createPolicy', () => {
    it('should create a new policy', async () => {
      const newPolicy = {
        name: 'Test Policy',
        description: 'Test description',
        permissions: [
          {
            service: 'admin' as const,
            actions: ['read'],
          },
        ],
      }

      const mockResponse = {
        id: '2',
        ...newPolicy,
        created_at: '2024-01-01',
        updated_at: '2024-01-01',
      }

      mockedAxios.post = jest.fn().mockResolvedValue({ data: mockResponse })

      const result = await iamService.createPolicy(newPolicy)

      expect(result).toEqual(mockResponse)
    })
  })

  describe('updatePolicy', () => {
    it('should update an existing policy', async () => {
      const updates = {
        description: 'Updated description',
      }

      const mockResponse = {
        id: '1',
        name: 'Test Policy',
        description: 'Updated description',
        permissions: [],
        created_at: '2024-01-01',
        updated_at: '2024-01-02',
      }

      mockedAxios.put = jest.fn().mockResolvedValue({ data: mockResponse })

      const result = await iamService.updatePolicy('1', updates)

      expect(result).toEqual(mockResponse)
    })
  })

  describe('deletePolicy', () => {
    it('should delete a policy', async () => {
      mockedAxios.delete = jest.fn().mockResolvedValue({})

      await iamService.deletePolicy('1')

      expect(mockedAxios.delete).toHaveBeenCalledWith('/iam/policies/1')
    })
  })

  describe('listUsers', () => {
    it('should list all IAM users', async () => {
      const mockUsers = {
        users: [
          {
            id: '1',
            email: 'user@test.com',
            name: 'Test User',
            policies: ['policy-1'],
            created_at: '2024-01-01',
            status: 'active' as const,
          },
        ],
        total: 1,
      }

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockUsers })

      const result = await iamService.listUsers()

      expect(result).toEqual(mockUsers)
    })

    it('should list users with filters', async () => {
      mockedAxios.get = jest.fn().mockResolvedValue({ data: { users: [], total: 0 } })

      await iamService.listUsers({
        status: 'active',
        limit: 10,
        offset: 0,
      })

      expect(mockedAxios.get).toHaveBeenCalledWith(
        '/iam/users',
        expect.objectContaining({
          params: {
            status: 'active',
            limit: 10,
            offset: 0,
          },
        })
      )
    })
  })

  describe('createUser', () => {
    it('should create a new IAM user', async () => {
      const newUser = {
        email: 'newuser@test.com',
        name: 'New User',
        password: 'password123',
        policies: ['policy-1'],
      }

      const mockResponse = {
        id: '2',
        email: newUser.email,
        name: newUser.name,
        policies: newUser.policies,
        created_at: '2024-01-01',
        status: 'active' as const,
      }

      mockedAxios.post = jest.fn().mockResolvedValue({ data: mockResponse })

      const result = await iamService.createUser(newUser)

      expect(result).toEqual(mockResponse)
    })
  })

  describe('updateUser', () => {
    it('should update an IAM user', async () => {
      const updates = {
        name: 'Updated Name',
      }

      const mockResponse = {
        id: '1',
        email: 'user@test.com',
        name: 'Updated Name',
        policies: [],
        created_at: '2024-01-01',
        status: 'active' as const,
      }

      mockedAxios.put = jest.fn().mockResolvedValue({ data: mockResponse })

      const result = await iamService.updateUser('1', updates)

      expect(result).toEqual(mockResponse)
    })
  })

  describe('deleteUser', () => {
    it('should delete an IAM user', async () => {
      mockedAxios.delete = jest.fn().mockResolvedValue({})

      await iamService.deleteUser('1')

      expect(mockedAxios.delete).toHaveBeenCalledWith('/iam/users/1')
    })
  })

  describe('assignPolicy', () => {
    it('should assign a policy to a user', async () => {
      mockedAxios.post = jest.fn().mockResolvedValue({})

      await iamService.assignPolicy('user-1', 'policy-1')

      expect(mockedAxios.post).toHaveBeenCalledWith('/iam/users/user-1/policies', {
        policy_id: 'policy-1',
      })
    })
  })

  describe('removePolicy', () => {
    it('should remove a policy from a user', async () => {
      mockedAxios.delete = jest.fn().mockResolvedValue({})

      await iamService.removePolicy('user-1', 'policy-1')

      expect(mockedAxios.delete).toHaveBeenCalledWith('/iam/users/user-1/policies/policy-1')
    })
  })

  describe('updateUserStatus', () => {
    it('should update user status', async () => {
      mockedAxios.patch = jest.fn().mockResolvedValue({})

      await iamService.updateUserStatus('user-1', 'inactive')

      expect(mockedAxios.patch).toHaveBeenCalledWith('/iam/users/user-1/status', {
        status: 'inactive',
      })
    })
  })
})
