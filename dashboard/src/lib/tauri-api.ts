import { ModelInfo, SystemInfo, MetricsSnapshot, InferenceParams, InfernoMetrics, ActiveProcessInfo, AppSettings, Notification, BatchJob, ApiKey, SecurityEvent, SecurityMetrics, CreateApiKeyRequest, CreateApiKeyResponse, NativeNotificationPayload } from '../types/inferno';

// Check if we're in a Tauri environment
const isTauri = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__;

// Import Tauri invoke - will be undefined in browser mode
let invoke: any = undefined;
try {
  if (isTauri) {
    // @ts-ignore - Dynamic import for Tauri environment only
    const { invoke: tauriInvoke } = require('@tauri-apps/api/core');
    invoke = tauriInvoke;
  }
} catch (e) {
  // Running in browser mode - invoke will remain undefined
  console.log('[Tauri API] Running in browser mode - using mock data');
}

// Safe invoke wrapper that throws in browser mode
async function safeInvoke<T>(command: string, args?: any): Promise<T> {
  if (!isTauri || !invoke) {
    throw new Error(`Tauri command '${command}' not available in browser mode`);
  }
  return await invoke(command, args);
}

// Mock data for browser mode
const mockModels: ModelInfo[] = [
  {
    id: 'llama-2-7b-chat',
    name: 'Llama 2 7B Chat',
    format: 'gguf',
    size: 4000000000,
    path: './models/llama-2-7b-chat.gguf',
    checksum: 'sha256:abc123def456',
    status: 'available',
    metadata: {
      architecture: 'llama',
      parameters: 7000000000,
      context_length: 4096,
      created_at: '2023-07-18T14:20:00Z',
      description: 'Llama 2 7B parameter chat model'
    }
  },
  {
    id: 'whisper-base',
    name: 'Whisper Base',
    format: 'onnx',
    size: 290000000,
    path: './models/whisper-base.onnx',
    checksum: 'sha256:def789ghi012',
    status: 'available',
    metadata: {
      architecture: 'whisper',
      parameters: 74000000,
      context_length: 448,
      created_at: '2023-01-10T12:00:00Z',
      description: 'Whisper base model for speech recognition'
    }
  }
];

const mockSystemInfo: SystemInfo = {
  cpu_name: 'Mock CPU',
  cpu_usage: 45.2,
  cpu_cores: 8,
  total_memory: 16 * 1024 * 1024 * 1024, // 16GB in bytes
  used_memory: 8 * 1024 * 1024 * 1024, // 8GB in bytes
  available_memory: 8 * 1024 * 1024 * 1024, // 8GB in bytes
  platform: 'darwin',
  arch: 'arm64'
};

const mockInfernoMetrics: InfernoMetrics = {
  inference_count: 1250,
  success_count: 1180,
  error_count: 70,
  average_latency: 245.5,
  cpu_usage: 45.2,
  memory_usage: 8 * 1024 * 1024 * 1024,
  gpu_usage: 65.3,
  active_models: 1,
  active_inferences: 2,
  active_streaming_sessions: 0
};

const mockActiveProcesses: ActiveProcessInfo = {
  active_models: [],
  active_inferences: 0,
  batch_jobs: 2,
  streaming_sessions: 0
};

// Tauri API service layer for communicating with the Rust backend
export class TauriApiService {

  // Model management
  async getModels(): Promise<ModelInfo[]> {
    try {
      if (isTauri && invoke) {
        return await invoke('get_models');
      } else {
        // Return mock data for browser mode
        return new Promise(resolve =>
          setTimeout(() => resolve(mockModels), 500)
        );
      }
    } catch (error) {
      console.error('Failed to get models:', error);
      // Fallback to mock data on error
      return mockModels;
    }
  }

  async loadModel(modelName: string, backendType: string): Promise<string> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would load model: ${modelName} (${backendType})`);
      return Promise.resolve(`mock-backend-${Date.now()}`);
    }
    try {
      return await safeInvoke('load_model', {
        model_name: modelName,
        backend_type: backendType
      });
    } catch (error) {
      console.error('Failed to load model:', error);
      throw new Error(`Failed to load model: ${error}`);
    }
  }

  async unloadModel(backendId: string): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would unload model: ${backendId}`);
      return Promise.resolve();
    }
    try {
      await safeInvoke('unload_model', { backend_id: backendId });
    } catch (error) {
      console.error('Failed to unload model:', error);
      throw new Error(`Failed to unload model: ${error}`);
    }
  }

  async getLoadedModels(): Promise<string[]> {
    try {
      if (isTauri && invoke) {
        return await invoke('get_loaded_models');
      } else {
        // Return mock loaded models for browser mode
        return new Promise(resolve =>
          setTimeout(() => resolve(['llama-2-7b-chat']), 300)
        );
      }
    } catch (error) {
      console.error('Failed to get loaded models:', error);
      // Fallback to mock data on error
      return ['llama-2-7b-chat'];
    }
  }

  // Inference
  async infer(backendId: string, prompt: string, params: InferenceParams): Promise<string> {
    if (!isTauri) {
      console.log(`[Browser Mode] Mock inference: "${prompt.substring(0, 50)}..."`);
      return Promise.resolve(`Mock response to: "${prompt}"`);
    }
    try {
      return await safeInvoke('infer', {
        backend_id: backendId,
        prompt,
        params
      });
    } catch (error) {
      console.error('Failed to run inference:', error);
      throw new Error(`Failed to run inference: ${error}`);
    }
  }

  // Streaming Inference
  async inferStream(backendId: string, prompt: string, params: InferenceParams): Promise<string> {
    if (!isTauri) {
      console.log(`[Browser Mode] Mock streaming inference: "${prompt.substring(0, 50)}..."`);
      return Promise.resolve(`stream-${Date.now()}`);
    }
    try {
      return await safeInvoke('infer_stream', {
        backend_id: backendId,
        prompt,
        params
      });
    } catch (error) {
      console.error('Failed to start streaming inference:', error);
      throw new Error(`Failed to start streaming inference: ${error}`);
    }
  }

  async sendNativeNotification(payload: NativeNotificationPayload): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would send notification: ${payload.title}`);
      return Promise.resolve();
    }

    try {
      await safeInvoke('send_native_notification', payload);
    } catch (error) {
      console.error('Failed to send native notification:', error);
      throw new Error(`Failed to send native notification: ${error}`);
    }
  }

  // Metrics and monitoring
  async getMetrics(): Promise<MetricsSnapshot> {
    if (!isTauri) {
      return Promise.resolve({
        inference_count: mockInfernoMetrics.inference_count,
        success_count: mockInfernoMetrics.success_count,
        error_count: mockInfernoMetrics.error_count,
        average_latency: mockInfernoMetrics.average_latency,
        models_loaded: mockInfernoMetrics.active_models,
      });
    }
    try {
      return await safeInvoke('get_metrics');
    } catch (error) {
      console.error('Failed to get metrics:', error);
      return {
        inference_count: mockInfernoMetrics.inference_count,
        success_count: mockInfernoMetrics.success_count,
        error_count: mockInfernoMetrics.error_count,
        average_latency: mockInfernoMetrics.average_latency,
        models_loaded: mockInfernoMetrics.active_models,
      };
    }
  }

  async getInfernoMetrics(): Promise<InfernoMetrics> {
    if (!isTauri) {
      return Promise.resolve(mockInfernoMetrics);
    }
    try {
      return await safeInvoke('get_inferno_metrics');
    } catch (error) {
      console.error('Failed to get Inferno metrics:', error);
      return mockInfernoMetrics;
    }
  }

  async getActiveProcesses(): Promise<ActiveProcessInfo> {
    if (!isTauri) {
      return Promise.resolve(mockActiveProcesses);
    }
    try {
      return await safeInvoke('get_active_processes');
    } catch (error) {
      console.error('Failed to get active processes:', error);
      return mockActiveProcesses;
    }
  }

  // System information
  async getSystemInfo(): Promise<SystemInfo> {
    if (!isTauri) {
      return Promise.resolve(mockSystemInfo);
    }
    try {
      return await safeInvoke('get_system_info');
    } catch (error) {
      console.error('Failed to get system info:', error);
      return mockSystemInfo; // Fallback to mock data
    }
  }

  // Model validation
  async validateModel(modelPath: string): Promise<boolean> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would validate model: ${modelPath}`);
      return Promise.resolve(true);
    }
    try {
      return await safeInvoke('validate_model', { model_path: modelPath });
    } catch (error) {
      console.error('Failed to validate model:', error);
      return false;
    }
  }

  // File dialogs
  async openFileDialog(): Promise<string | null> {
    if (!isTauri) {
      console.log('[Browser Mode] File dialog not available');
      return Promise.resolve(null);
    }
    try {
      return await safeInvoke('open_file_dialog');
    } catch (error) {
      console.error('Failed to open file dialog:', error);
      return null;
    }
  }

  // File upload
  async uploadModel(sourcePath: string, targetName?: string): Promise<string> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would upload model: ${sourcePath}`);
      return Promise.resolve(`/mock/models/${targetName || 'uploaded-model.gguf'}`);
    }
    try {
      return await safeInvoke('upload_model', {
        source_path: sourcePath,
        target_name: targetName
      });
    } catch (error) {
      console.error('Failed to upload model:', error);
      throw new Error(`Failed to upload model: ${error}`);
    }
  }

  // Activity logging
  async getRecentActivities(limit?: number): Promise<any[]> {
    if (!isTauri) {
      return Promise.resolve([
        {
          id: '1',
          activity_type: 'inference',
          title: 'Inference Completed',
          description: 'llama-2-7b-chat processed query successfully',
          status: 'success',
          user: 'system',
          timestamp: new Date().toISOString()
        },
        {
          id: '2',
          activity_type: 'modelload',
          title: 'Model Loaded',
          description: 'whisper-base loaded and ready',
          status: 'success',
          user: 'system',
          timestamp: new Date(Date.now() - 3600000).toISOString()
        },
        {
          id: '3',
          activity_type: 'modelupload',
          title: 'Model Uploaded',
          description: 'New model uploaded to repository',
          status: 'success',
          user: 'admin',
          timestamp: new Date(Date.now() - 7200000).toISOString()
        }
      ]);
    }
    try {
      return await safeInvoke('get_recent_activities', { limit });
    } catch (error) {
      console.error('Failed to get recent activities:', error);
      return [];
    }
  }

  async getActivityStats(): Promise<any> {
    if (!isTauri) {
      return Promise.resolve({ total: 125, today: 15, this_week: 67 });
    }
    try {
      return await safeInvoke('get_activity_stats');
    } catch (error) {
      console.error('Failed to get activity stats:', error);
      return { total: 0, today: 0, this_week: 0 };
    }
  }

  async clearActivities(): Promise<void> {
    if (!isTauri) {
      console.log('[Browser Mode] Would clear activities');
      return Promise.resolve();
    }
    try {
      await safeInvoke('clear_activities');
    } catch (error) {
      console.error('Failed to clear activities:', error);
      throw new Error(`Failed to clear activities: ${error}`);
    }
  }

  // Settings management
  async getSettings(): Promise<AppSettings> {
    if (!isTauri) {
      return Promise.resolve({
        modelsDirectory: './models',
        autoDiscoverModels: true,
        defaultTemperature: 0.7,
        defaultMaxTokens: 2048,
        defaultTopP: 0.9,
        defaultTopK: 40,
        maxMemoryUsage: 16000000000,
        preferGPU: true,
        maxConcurrentInferences: 4,
        enableCache: true,
        cacheDirectory: './cache',
        maxCacheSize: 10000000000,
        enableRestAPI: true,
        apiPort: 8080,
        enableCORS: true,
        requireAuthentication: false,
        enableAuditLog: true,
        logLevel: 'info',
        notifications: {
          enabled: true,
          types: {
            system: true,
            inference: true,
            security: true,
            batch: true,
            model: true,
          },
          priority_filter: 'high',
          auto_dismiss_after: 5000,
          max_notifications: 10
        }
      });
    }
    try {
      return await safeInvoke('get_settings');
    } catch (error) {
      console.error('Failed to get settings:', error);
      throw new Error(`Failed to get settings: ${error}`);
    }
  }

  async setSettings(settings: AppSettings): Promise<void> {
    try {
      // Convert camelCase to snake_case for Rust backend
      const rustSettings = {
        models_directory: settings.modelsDirectory,
        auto_discover_models: settings.autoDiscoverModels,
        default_temperature: settings.defaultTemperature,
        default_max_tokens: settings.defaultMaxTokens,
        default_top_p: settings.defaultTopP,
        default_top_k: settings.defaultTopK,
        max_memory_usage: settings.maxMemoryUsage,
        prefer_gpu: settings.preferGPU,
        max_concurrent_inferences: settings.maxConcurrentInferences,
        enable_cache: settings.enableCache,
        cache_directory: settings.cacheDirectory,
        max_cache_size: settings.maxCacheSize,
        enable_rest_api: settings.enableRestAPI,
        api_port: settings.apiPort,
        enable_cors: settings.enableCORS,
        require_authentication: settings.requireAuthentication,
        enable_audit_log: settings.enableAuditLog,
        log_level: settings.logLevel,
        notifications: {
          enabled: settings.notifications.enabled,
          types: settings.notifications.types,
          priority_filter: settings.notifications.priority_filter,
          auto_dismiss_after: settings.notifications.auto_dismiss_after,
          max_notifications: settings.notifications.max_notifications,
        },
      };
      await invoke('set_settings', { settings: rustSettings });
    } catch (error) {
      console.error('Failed to set settings:', error);
      throw new Error(`Failed to set settings: ${error}`);
    }
  }

  // Notification management
  async getNotifications(): Promise<Notification[]> {
    if (!isTauri) {
      return Promise.resolve([]);
    }
    try {
      return await safeInvoke('get_notifications');
    } catch (error) {
      console.error('Failed to get notifications:', error);
      return [];
    }
  }

  async getUnreadNotificationCount(): Promise<number> {
    if (!isTauri) {
      return Promise.resolve(0);
    }
    try {
      return await safeInvoke('get_unread_notification_count');
    } catch (error) {
      console.error('Failed to get unread notification count:', error);
      return 0;
    }
  }

  async markNotificationAsRead(notificationId: string): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would mark notification as read: ${notificationId}`);
      return Promise.resolve();
    }
    try {
      await safeInvoke('mark_notification_as_read', { notification_id: notificationId });
    } catch (error) {
      console.error('Failed to mark notification as read:', error);
      throw new Error(`Failed to mark notification as read: ${error}`);
    }
  }

  async markAllNotificationsAsRead(): Promise<void> {
    if (!isTauri) {
      console.log('[Browser Mode] Would mark all notifications as read');
      return Promise.resolve();
    }
    try {
      await safeInvoke('mark_all_notifications_as_read');
    } catch (error) {
      console.error('Failed to mark all notifications as read:', error);
      throw new Error(`Failed to mark all notifications as read: ${error}`);
    }
  }

  async dismissNotification(notificationId: string): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would dismiss notification: ${notificationId}`);
      return Promise.resolve();
    }
    try {
      await safeInvoke('dismiss_notification', { notification_id: notificationId });
    } catch (error) {
      console.error('Failed to dismiss notification:', error);
      throw new Error(`Failed to dismiss notification: ${error}`);
    }
  }

  async clearAllNotifications(): Promise<void> {
    if (!isTauri) {
      console.log('[Browser Mode] Would clear all notifications');
      return Promise.resolve();
    }
    try {
      await safeInvoke('clear_all_notifications');
    } catch (error) {
      console.error('Failed to clear all notifications:', error);
      throw new Error(`Failed to clear all notifications: ${error}`);
    }
  }

  async createNotification(notification: Omit<Notification, 'id' | 'timestamp'>): Promise<string> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would create notification: ${notification.title}`);
      return Promise.resolve(`mock-notification-${Date.now()}`);
    }
    try {
      return await safeInvoke('create_notification', { notification });
    } catch (error) {
      console.error('Failed to create notification:', error);
      throw new Error(`Failed to create notification: ${error}`);
    }
  }

  // Batch job management
  async getBatchJobs(): Promise<BatchJob[]> {
    if (!isTauri) {
      return Promise.resolve([]);
    }
    try {
      return await safeInvoke('get_batch_jobs');
    } catch (error) {
      console.error('Failed to get batch jobs:', error);
      return [];
    }
  }

  async getBatchJob(jobId: string): Promise<BatchJob | null> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would get batch job: ${jobId}`);
      return Promise.resolve(null);
    }
    try {
      return await safeInvoke('get_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to get batch job:', error);
      return null;
    }
  }

  async createBatchJob(jobData: any): Promise<string> {
    if (!isTauri) {
      console.log('[Browser Mode] Would create batch job');
      return Promise.resolve(`mock-job-${Date.now()}`);
    }
    try {
      return await safeInvoke('create_batch_job', { job_data: jobData });
    } catch (error) {
      console.error('Failed to create batch job:', error);
      throw new Error(`Failed to create batch job: ${error}`);
    }
  }

  async startBatchJob(jobId: string): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would start batch job: ${jobId}`);
      return Promise.resolve();
    }
    try {
      await safeInvoke('start_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to start batch job:', error);
      throw new Error(`Failed to start batch job: ${error}`);
    }
  }

  async pauseBatchJob(jobId: string): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would pause batch job: ${jobId}`);
      return Promise.resolve();
    }
    try {
      await safeInvoke('pause_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to pause batch job:', error);
      throw new Error(`Failed to pause batch job: ${error}`);
    }
  }

  async cancelBatchJob(jobId: string): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would cancel batch job: ${jobId}`);
      return Promise.resolve();
    }
    try {
      await safeInvoke('cancel_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to cancel batch job:', error);
      throw new Error(`Failed to cancel batch job: ${error}`);
    }
  }

  async deleteBatchJob(jobId: string): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would delete batch job: ${jobId}`);
      return Promise.resolve();
    }
    try {
      await safeInvoke('delete_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to delete batch job:', error);
      throw new Error(`Failed to delete batch job: ${error}`);
    }
  }

  async getBatchJobCount(): Promise<number> {
    if (!isTauri) {
      return Promise.resolve(mockActiveProcesses.batch_jobs);
    }
    try {
      return await safeInvoke('get_batch_job_count');
    } catch (error) {
      console.error('Failed to get batch job count:', error);
      return mockActiveProcesses.batch_jobs;
    }
  }

  async getActiveBatchJobCount(): Promise<number> {
    if (!isTauri) {
      return Promise.resolve(1); // 1 active job out of 2 total
    }
    try {
      return await safeInvoke('get_active_batch_job_count');
    } catch (error) {
      console.error('Failed to get active batch job count:', error);
      return 1;
    }
  }

  // Search functionality
  async searchAll(query: string, limit?: number): Promise<any> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would search for: ${query}`);
      return Promise.resolve({ models: [], jobs: [], activities: [] });
    }
    try {
      return await safeInvoke('search_all', { query, limit });
    } catch (error) {
      console.error('Failed to search:', error);
      return { models: [], jobs: [], activities: [] };
    }
  }

  // Security management
  async createApiKey(request: CreateApiKeyRequest): Promise<CreateApiKeyResponse> {
    if (!isTauri) {
      console.log('[Browser Mode] Would create API key');
      const rawKey = 'inferno_mock_key_' + Math.random().toString(36).substring(2);
      return Promise.resolve({
        api_key: {
          id: `mock-key-${Date.now()}`,
          name: request.name,
          permissions: request.permissions,
          created_at: new Date().toISOString(),
          expires_at: undefined,
          is_active: true,
          key_hash: 'mock_hash_' + rawKey.substring(0, 8),
          key_prefix: rawKey.substring(0, 8),
          usage_count: 0,
          created_by: 'test-user'
        },
        raw_key: rawKey
      });
    }
    try {
      return await safeInvoke('create_api_key', { request });
    } catch (error) {
      console.error('Failed to create API key:', error);
      throw new Error(`Failed to create API key: ${error}`);
    }
  }

  async getApiKeys(): Promise<ApiKey[]> {
    if (!isTauri) {
      return Promise.resolve([]);
    }
    try {
      return await safeInvoke('get_api_keys');
    } catch (error) {
      console.error('Failed to get API keys:', error);
      return [];
    }
  }

  async revokeApiKey(keyId: string): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would revoke API key: ${keyId}`);
      return Promise.resolve();
    }
    try {
      await safeInvoke('revoke_api_key', { key_id: keyId });
    } catch (error) {
      console.error('Failed to revoke API key:', error);
      throw new Error(`Failed to revoke API key: ${error}`);
    }
  }

  async deleteApiKey(keyId: string): Promise<void> {
    if (!isTauri) {
      console.log(`[Browser Mode] Would delete API key: ${keyId}`);
      return Promise.resolve();
    }
    try {
      await safeInvoke('delete_api_key', { key_id: keyId });
    } catch (error) {
      console.error('Failed to delete API key:', error);
      throw new Error(`Failed to delete API key: ${error}`);
    }
  }

  async validateApiKey(rawKey: string): Promise<ApiKey | null> {
    if (!isTauri) {
      console.log('[Browser Mode] Would validate API key');
      return Promise.resolve(null);
    }
    try {
      return await safeInvoke('validate_api_key', { raw_key: rawKey });
    } catch (error) {
      console.error('Failed to validate API key:', error);
      return null;
    }
  }

  async getSecurityEvents(limit?: number): Promise<SecurityEvent[]> {
    if (!isTauri) {
      return Promise.resolve([]);
    }
    try {
      return await safeInvoke('get_security_events', { limit });
    } catch (error) {
      console.error('Failed to get security events:', error);
      return [];
    }
  }

  async getSecurityMetrics(): Promise<SecurityMetrics> {
    if (!isTauri) {
      return Promise.resolve({} as SecurityMetrics);
    }
    try {
      return await safeInvoke('get_security_metrics');
    } catch (error) {
      console.error('Failed to get security metrics:', error);
      return {} as SecurityMetrics;
    }
  }

  async clearSecurityEvents(): Promise<void> {
    if (!isTauri) {
      console.log('[Browser Mode] Would clear security events');
      return Promise.resolve();
    }
    try {
      await safeInvoke('clear_security_events');
    } catch (error) {
      console.error('Failed to clear security events:', error);
      throw new Error(`Failed to clear security events: ${error}`);
    }
  }
}

// Singleton instance
export const tauriApi = new TauriApiService();
