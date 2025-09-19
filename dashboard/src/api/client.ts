import axios, { AxiosInstance, AxiosResponse } from 'axios';
import {
  ModelInfo,
  InferenceRequest,
  InferenceResponse,
  SystemMetrics,
  BatchJob,
  AuditLog,
  Tenant,
  ApiKey
} from '@/types/inferno';

class InfernoClient {
  private api: AxiosInstance;

  constructor(baseURL: string = process.env.INFERNO_API_URL || 'http://localhost:8080') {
    this.api = axios.create({
      baseURL,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Request interceptor for auth
    this.api.interceptors.request.use((config) => {
      const token = localStorage.getItem('inferno_auth_token');
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    });

    // Response interceptor for error handling
    this.api.interceptors.response.use(
      (response) => response,
      (error) => {
        console.error('API Error:', error);
        return Promise.reject(error);
      }
    );
  }

  // Models API
  async getModels(): Promise<ModelInfo[]> {
    const response = await this.api.get('/api/models');
    return response.data;
  }

  async getModel(id: string): Promise<ModelInfo> {
    const response = await this.api.get(`/api/models/${id}`);
    return response.data;
  }

  async uploadModel(file: File, onProgress?: (progress: number) => void): Promise<ModelInfo> {
    const formData = new FormData();
    formData.append('model', file);

    const response = await this.api.post('/api/models/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
      onUploadProgress: (progressEvent) => {
        if (onProgress && progressEvent.total) {
          const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total);
          onProgress(progress);
        }
      },
    });

    return response.data;
  }

  async deleteModel(id: string): Promise<void> {
    await this.api.delete(`/api/models/${id}`);
  }

  async loadModel(id: string): Promise<void> {
    await this.api.post(`/api/models/${id}/load`);
  }

  async unloadModel(id: string): Promise<void> {
    await this.api.post(`/api/models/${id}/unload`);
  }

  async quantizeModel(id: string, options: any): Promise<{ job_id: string }> {
    const response = await this.api.post(`/api/models/${id}/quantize`, options);
    return response.data;
  }

  // Inference API
  async runInference(request: InferenceRequest): Promise<InferenceResponse> {
    const response = await this.api.post('/api/inference', request);
    return response.data;
  }

  async *streamInference(request: InferenceRequest): AsyncGenerator<string> {
    const response = await fetch(`${this.api.defaults.baseURL}/api/inference/stream`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.body) {
      throw new Error('No response body');
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder();

    try {
      while (true) {
        const { done, value } = await reader.read();

        if (done) break;

        const chunk = decoder.decode(value);
        const lines = chunk.split('\n');

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6);
            if (data === '[DONE]') return;

            try {
              const parsed = JSON.parse(data);
              if (parsed.token) {
                yield parsed.token;
              }
            } catch (e) {
              // Skip invalid JSON
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  }

  // Metrics API
  async getSystemMetrics(): Promise<SystemMetrics> {
    const response = await this.api.get('/api/metrics');
    return response.data;
  }

  async getModelMetrics(id: string): Promise<any> {
    const response = await this.api.get(`/api/metrics/models/${id}`);
    return response.data;
  }

  async getMetricsHistory(range: string = '24h'): Promise<SystemMetrics[]> {
    const response = await this.api.get(`/api/metrics/history?range=${range}`);
    return response.data;
  }

  // Batch Jobs API
  async getBatchJobs(): Promise<BatchJob[]> {
    const response = await this.api.get('/api/jobs');
    return response.data;
  }

  async getBatchJob(id: string): Promise<BatchJob> {
    const response = await this.api.get(`/api/jobs/${id}`);
    return response.data;
  }

  async createBatchJob(job: Partial<BatchJob>): Promise<BatchJob> {
    const response = await this.api.post('/api/jobs', job);
    return response.data;
  }

  async cancelBatchJob(id: string): Promise<void> {
    await this.api.post(`/api/jobs/${id}/cancel`);
  }

  async deleteBatchJob(id: string): Promise<void> {
    await this.api.delete(`/api/jobs/${id}`);
  }

  // Security & Audit API
  async getAuditLogs(filters?: any): Promise<AuditLog[]> {
    const response = await this.api.get('/api/audit', { params: filters });
    return response.data;
  }

  async getSecurityEvents(filters?: any): Promise<any[]> {
    const response = await this.api.get('/api/security/events', { params: filters });
    return response.data;
  }

  // Multi-tenancy API
  async getTenants(): Promise<Tenant[]> {
    const response = await this.api.get('/api/tenants');
    return response.data;
  }

  async createTenant(tenant: Partial<Tenant>): Promise<Tenant> {
    const response = await this.api.post('/api/tenants', tenant);
    return response.data;
  }

  async updateTenant(id: string, updates: Partial<Tenant>): Promise<Tenant> {
    const response = await this.api.put(`/api/tenants/${id}`, updates);
    return response.data;
  }

  // API Keys
  async getApiKeys(): Promise<ApiKey[]> {
    const response = await this.api.get('/api/keys');
    return response.data;
  }

  async createApiKey(data: { name: string; permissions: string[] }): Promise<ApiKey & { key: string }> {
    const response = await this.api.post('/api/keys', data);
    return response.data;
  }

  async revokeApiKey(id: string): Promise<void> {
    await this.api.delete(`/api/keys/${id}`);
  }

  // System Configuration
  async getConfig(): Promise<any> {
    const response = await this.api.get('/api/config');
    return response.data;
  }

  async updateConfig(config: any): Promise<any> {
    const response = await this.api.put('/api/config', config);
    return response.data;
  }

  // Health Check
  async getHealth(): Promise<{ status: string; version: string; uptime: number }> {
    const response = await this.api.get('/api/health');
    return response.data;
  }
}

// Singleton instance
export const infernoClient = new InfernoClient();

// React Query keys
export const queryKeys = {
  models: ['models'] as const,
  model: (id: string) => ['models', id] as const,
  metrics: ['metrics'] as const,
  metricsHistory: (range: string) => ['metrics', 'history', range] as const,
  batchJobs: ['batch-jobs'] as const,
  batchJob: (id: string) => ['batch-jobs', id] as const,
  auditLogs: (filters?: any) => ['audit-logs', filters] as const,
  tenants: ['tenants'] as const,
  apiKeys: ['api-keys'] as const,
  config: ['config'] as const,
  health: ['health'] as const,
};