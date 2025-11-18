import apiClient from './client';

export interface IAMPolicy {
  id: string;
  name: string;
  description: string;
  permissions: Permission[];
  created_at: string;
  updated_at: string;
}

export interface Permission {
  service: 'onboarding' | 'admin' | 'billing' | 'analytics' | 'iam';
  actions: string[];
  resources?: string[];
}

export interface IAMUser {
  id: string;
  email: string;
  name: string;
  policies: string[];
  created_at: string;
  last_login?: string;
  status: 'active' | 'inactive' | 'suspended';
}

export interface CreateIAMUserRequest {
  email: string;
  name: string;
  password: string;
  policies: string[];
}

export const iamService = {
  async listPolicies(): Promise<IAMPolicy[]> {
    const response = await apiClient.get('/iam/policies');
    return response.data;
  },

  async getPolicy(id: string): Promise<IAMPolicy> {
    const response = await apiClient.get(`/iam/policies/${id}`);
    return response.data;
  },

  async createPolicy(policy: Omit<IAMPolicy, 'id' | 'created_at' | 'updated_at'>): Promise<IAMPolicy> {
    const response = await apiClient.post('/iam/policies', policy);
    return response.data;
  },

  async updatePolicy(id: string, updates: Partial<IAMPolicy>): Promise<IAMPolicy> {
    const response = await apiClient.put(`/iam/policies/${id}`, updates);
    return response.data;
  },

  async deletePolicy(id: string): Promise<void> {
    await apiClient.delete(`/iam/policies/${id}`);
  },

  async listUsers(params?: {
    status?: string;
    limit?: number;
    offset?: number;
  }): Promise<{ users: IAMUser[]; total: number }> {
    const response = await apiClient.get('/iam/users', { params });
    return response.data;
  },

  async getUser(id: string): Promise<IAMUser> {
    const response = await apiClient.get(`/iam/users/${id}`);
    return response.data;
  },

  async createUser(user: CreateIAMUserRequest): Promise<IAMUser> {
    const response = await apiClient.post('/iam/users', user);
    return response.data;
  },

  async updateUser(id: string, updates: Partial<IAMUser>): Promise<IAMUser> {
    const response = await apiClient.put(`/iam/users/${id}`, updates);
    return response.data;
  },

  async deleteUser(id: string): Promise<void> {
    await apiClient.delete(`/iam/users/${id}`);
  },

  async assignPolicy(userId: string, policyId: string): Promise<void> {
    await apiClient.post(`/iam/users/${userId}/policies`, { policy_id: policyId });
  },

  async removePolicy(userId: string, policyId: string): Promise<void> {
    await apiClient.delete(`/iam/users/${userId}/policies/${policyId}`);
  },

  async updateUserStatus(userId: string, status: IAMUser['status']): Promise<void> {
    await apiClient.patch(`/iam/users/${userId}/status`, { status });
  },
};

export default iamService;
