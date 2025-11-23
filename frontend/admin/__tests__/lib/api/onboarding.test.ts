import axios from 'axios'
import { onboardingService } from '@/lib/api/onboarding'

jest.mock('axios')
const mockedAxios = axios as jest.Mocked<typeof axios>

describe('Onboarding Service', () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  describe('bulkImportCSV', () => {
    it('should upload CSV file successfully', async () => {
      const mockFile = new File(['test'], 'test.csv', { type: 'text/csv' })
      const mockResponse = {
        data: {
          job_id: 'job-123',
          status: 'pending',
          total_records: 100,
          processed_records: 0,
          failed_records: 0,
          created_at: '2024-01-01',
        },
      }

      mockedAxios.post = jest.fn().mockResolvedValue(mockResponse)

      const result = await onboardingService.bulkImportCSV(mockFile, {
        role_type: 'student',
      })

      expect(result).toEqual(mockResponse.data)
      expect(mockedAxios.post).toHaveBeenCalledWith(
        '/onboarding/bulk-import/csv',
        expect.any(FormData),
        expect.objectContaining({
          headers: {
            'Content-Type': 'multipart/form-data',
          },
        })
      )
    })
  })

  describe('bulkImportAPI', () => {
    it('should import users via API', async () => {
      const mockUsers = [
        { email: 'user1@test.com', name: 'User 1', student_id: '001' },
        { email: 'user2@test.com', name: 'User 2', student_id: '002' },
      ]

      const mockResponse = {
        data: {
          job_id: 'job-456',
          status: 'processing',
          total_records: 2,
          processed_records: 0,
          failed_records: 0,
          created_at: '2024-01-01',
        },
      }

      mockedAxios.post = jest.fn().mockResolvedValue(mockResponse)

      const result = await onboardingService.bulkImportAPI(mockUsers, {
        role_type: 'student',
        auto_provision: true,
      })

      expect(result).toEqual(mockResponse.data)
    })
  })

  describe('getJob', () => {
    it('should get job details', async () => {
      const mockJob = {
        id: 'job-123',
        status: 'completed',
        method: 'csv',
        total_records: 100,
        processed_records: 100,
        failed_records: 0,
        progress_percentage: 100,
        started_at: '2024-01-01T10:00:00Z',
        completed_at: '2024-01-01T10:05:00Z',
      }

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockJob })

      const result = await onboardingService.getJob('job-123')

      expect(result).toEqual(mockJob)
    })
  })

  describe('listJobs', () => {
    it('should list all jobs', async () => {
      const mockJobs = {
        jobs: [
          {
            id: 'job-1',
            status: 'completed',
            method: 'csv',
            total_records: 100,
            processed_records: 100,
            failed_records: 0,
            progress_percentage: 100,
            started_at: '2024-01-01',
          },
        ],
        total: 1,
      }

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockJobs })

      const result = await onboardingService.listJobs()

      expect(result).toEqual(mockJobs)
    })

    it('should list jobs with filters', async () => {
      mockedAxios.get = jest.fn().mockResolvedValue({ data: { jobs: [], total: 0 } })

      await onboardingService.listJobs({
        status: 'completed',
        limit: 10,
        offset: 0,
      })

      expect(mockedAxios.get).toHaveBeenCalledWith(
        '/onboarding/jobs',
        expect.objectContaining({
          params: {
            status: 'completed',
            limit: 10,
            offset: 0,
          },
        })
      )
    })
  })

  describe('cancelJob', () => {
    it('should cancel a job', async () => {
      mockedAxios.post = jest.fn().mockResolvedValue({})

      await onboardingService.cancelJob('job-123')

      expect(mockedAxios.post).toHaveBeenCalledWith('/onboarding/jobs/job-123/cancel')
    })
  })

  describe('getIntegrations', () => {
    it('should get all integrations', async () => {
      const mockIntegrations = [
        {
          id: '1',
          type: 'ldap',
          name: 'LDAP Integration',
          enabled: true,
          config: {},
          last_sync: '2024-01-01',
        },
      ]

      mockedAxios.get = jest.fn().mockResolvedValue({ data: mockIntegrations })

      const result = await onboardingService.getIntegrations()

      expect(result).toEqual(mockIntegrations)
    })
  })

  describe('syncIntegration', () => {
    it('should sync an integration', async () => {
      const mockResponse = {
        data: {
          job_id: 'sync-job-123',
          status: 'pending',
          total_records: 0,
          processed_records: 0,
          failed_records: 0,
          created_at: '2024-01-01',
        },
      }

      mockedAxios.post = jest.fn().mockResolvedValue(mockResponse)

      const result = await onboardingService.syncIntegration('integration-1')

      expect(result).toEqual(mockResponse.data)
      expect(mockedAxios.post).toHaveBeenCalledWith('/onboarding/integrations/integration-1/sync')
    })
  })

  describe('testIntegration', () => {
    it('should test integration config', async () => {
      const mockConfig = {
        host: 'ldap.example.com',
        port: 389,
      }

      const mockResponse = {
        data: {
          success: true,
          message: 'Connection successful',
        },
      }

      mockedAxios.post = jest.fn().mockResolvedValue(mockResponse)

      const result = await onboardingService.testIntegration(mockConfig)

      expect(result).toEqual(mockResponse.data)
    })
  })
})
