import apiClient from './client';

export interface BulkImportRequest {
  method: 'csv' | 'api' | 'ldap' | 'saml' | 'google' | 'microsoft';
  data?: any;
  file?: File;
  config?: {
    role_type?: 'student' | 'instructor';
    auto_provision?: boolean;
    send_welcome_email?: boolean;
  };
}

export interface BulkImportResponse {
  job_id: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  total_records: number;
  processed_records: number;
  failed_records: number;
  created_at: string;
}

export interface OnboardingJob {
  id: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  method: string;
  total_records: number;
  processed_records: number;
  failed_records: number;
  progress_percentage: number;
  started_at: string;
  completed_at?: string;
  errors?: string[];
}

export interface IntegrationConfig {
  id: string;
  type: 'ldap' | 'saml' | 'google_workspace' | 'microsoft_365';
  name: string;
  enabled: boolean;
  config: Record<string, any>;
  last_sync?: string;
}

export const onboardingService = {
  async bulkImportCSV(file: File, config: BulkImportRequest['config']): Promise<BulkImportResponse> {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('config', JSON.stringify(config));

    const response = await apiClient.post('/onboarding/bulk-import/csv', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    });
    return response.data;
  },

  async bulkImportAPI(users: any[], config: BulkImportRequest['config']): Promise<BulkImportResponse> {
    const response = await apiClient.post('/onboarding/bulk-import/api', {
      users,
      config,
    });
    return response.data;
  },

  async getJob(jobId: string): Promise<OnboardingJob> {
    const response = await apiClient.get(`/onboarding/jobs/${jobId}`);
    return response.data;
  },

  async listJobs(params?: {
    status?: string;
    limit?: number;
    offset?: number;
  }): Promise<{ jobs: OnboardingJob[]; total: number }> {
    const response = await apiClient.get('/onboarding/jobs', { params });
    return response.data;
  },

  async cancelJob(jobId: string): Promise<void> {
    await apiClient.post(`/onboarding/jobs/${jobId}/cancel`);
  },

  async getIntegrations(): Promise<IntegrationConfig[]> {
    const response = await apiClient.get('/onboarding/integrations');
    return response.data;
  },

  async createIntegration(integration: Omit<IntegrationConfig, 'id' | 'last_sync'>): Promise<IntegrationConfig> {
    const response = await apiClient.post('/onboarding/integrations', integration);
    return response.data;
  },

  async updateIntegration(id: string, updates: Partial<IntegrationConfig>): Promise<IntegrationConfig> {
    const response = await apiClient.put(`/onboarding/integrations/${id}`, updates);
    return response.data;
  },

  async deleteIntegration(id: string): Promise<void> {
    await apiClient.delete(`/onboarding/integrations/${id}`);
  },

  async syncIntegration(id: string): Promise<BulkImportResponse> {
    const response = await apiClient.post(`/onboarding/integrations/${id}/sync`);
    return response.data;
  },

  async testIntegration(config: IntegrationConfig['config']): Promise<{ success: boolean; message: string }> {
    const response = await apiClient.post('/onboarding/integrations/test', config);
    return response.data;
  },
};

export default onboardingService;
