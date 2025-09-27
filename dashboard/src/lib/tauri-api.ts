import { ModelInfo, SystemInfo, MetricsSnapshot, InferenceParams, InfernoMetrics, ActiveProcessInfo, AppSettings, Notification, BatchJob, ApiKey, SecurityEvent, SecurityMetrics, CreateApiKeyRequest, CreateApiKeyResponse } from '../types/inferno';

// Check if we're in a Tauri environment
const isTauri = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__;

// Conditionally import Tauri APIs
let invoke: any = null;
if (isTauri) {
  try {
    const tauriCore = require('@tauri-apps/api/core');
    invoke = tauriCore.invoke;
  } catch (error) {
    console.warn('Tauri APIs not available, running in browser mode');
  }
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
    try {
      return await invoke('load_model', {
        model_name: modelName,
        backend_type: backendType
      });
    } catch (error) {
      console.error('Failed to load model:', error);
      throw new Error(`Failed to load model: ${error}`);
    }
  }

  async unloadModel(backendId: string): Promise<void> {
    try {
      await invoke('unload_model', { backend_id: backendId });
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
    try {
      return await invoke('infer', {
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
    try {
      return await invoke('infer_stream', {
        backend_id: backendId,
        prompt,
        params
      });
    } catch (error) {
      console.error('Failed to start streaming inference:', error);
      throw new Error(`Failed to start streaming inference: ${error}`);
    }
  }

  // Metrics and monitoring
  async getMetrics(): Promise<MetricsSnapshot> {
    try {
      return await invoke('get_metrics');
    } catch (error) {
      console.error('Failed to get metrics:', error);
      throw new Error(`Failed to get metrics: ${error}`);
    }
  }

  async getInfernoMetrics(): Promise<InfernoMetrics> {
    try {
      return await invoke('get_inferno_metrics');
    } catch (error) {
      console.error('Failed to get Inferno metrics:', error);
      throw new Error(`Failed to get Inferno metrics: ${error}`);
    }
  }

  async getActiveProcesses(): Promise<ActiveProcessInfo> {
    try {
      return await invoke('get_active_processes');
    } catch (error) {
      console.error('Failed to get active processes:', error);
      throw new Error(`Failed to get active processes: ${error}`);
    }
  }

  // System information
  async getSystemInfo(): Promise<SystemInfo> {
    try {
      return await invoke('get_system_info');
    } catch (error) {
      console.error('Failed to get system info:', error);
      throw new Error(`Failed to get system info: ${error}`);
    }
  }

  // Model validation
  async validateModel(modelPath: string): Promise<boolean> {
    try {
      return await invoke('validate_model', { model_path: modelPath });
    } catch (error) {
      console.error('Failed to validate model:', error);
      return false;
    }
  }

  // File dialogs
  async openFileDialog(): Promise<string | null> {
    try {
      return await invoke('open_file_dialog');
    } catch (error) {
      console.error('Failed to open file dialog:', error);
      return null;
    }
  }

  // File upload
  async uploadModel(sourcePath: string, targetName?: string): Promise<string> {
    try {
      return await invoke('upload_model', {
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
    try {
      return await invoke('get_recent_activities', { limit });
    } catch (error) {
      console.error('Failed to get recent activities:', error);
      throw new Error(`Failed to get recent activities: ${error}`);
    }
  }

  async getActivityStats(): Promise<any> {
    try {
      return await invoke('get_activity_stats');
    } catch (error) {
      console.error('Failed to get activity stats:', error);
      throw new Error(`Failed to get activity stats: ${error}`);
    }
  }

  async clearActivities(): Promise<void> {
    try {
      await invoke('clear_activities');
    } catch (error) {
      console.error('Failed to clear activities:', error);
      throw new Error(`Failed to clear activities: ${error}`);
    }
  }

  // Settings management
  async getSettings(): Promise<AppSettings> {
    try {
      return await invoke('get_settings');
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
    try {
      return await invoke('get_notifications');
    } catch (error) {
      console.error('Failed to get notifications:', error);
      throw new Error(`Failed to get notifications: ${error}`);
    }
  }

  async getUnreadNotificationCount(): Promise<number> {
    try {
      return await invoke('get_unread_notification_count');
    } catch (error) {
      console.error('Failed to get unread notification count:', error);
      return 0;
    }
  }

  async markNotificationAsRead(notificationId: string): Promise<void> {
    try {
      await invoke('mark_notification_as_read', { notification_id: notificationId });
    } catch (error) {
      console.error('Failed to mark notification as read:', error);
      throw new Error(`Failed to mark notification as read: ${error}`);
    }
  }

  async markAllNotificationsAsRead(): Promise<void> {
    try {
      await invoke('mark_all_notifications_as_read');
    } catch (error) {
      console.error('Failed to mark all notifications as read:', error);
      throw new Error(`Failed to mark all notifications as read: ${error}`);
    }
  }

  async dismissNotification(notificationId: string): Promise<void> {
    try {
      await invoke('dismiss_notification', { notification_id: notificationId });
    } catch (error) {
      console.error('Failed to dismiss notification:', error);
      throw new Error(`Failed to dismiss notification: ${error}`);
    }
  }

  async clearAllNotifications(): Promise<void> {
    try {
      await invoke('clear_all_notifications');
    } catch (error) {
      console.error('Failed to clear all notifications:', error);
      throw new Error(`Failed to clear all notifications: ${error}`);
    }
  }

  async createNotification(notification: Omit<Notification, 'id' | 'timestamp'>): Promise<string> {
    try {
      return await invoke('create_notification', { notification });
    } catch (error) {
      console.error('Failed to create notification:', error);
      throw new Error(`Failed to create notification: ${error}`);
    }
  }

  // Batch job management
  async getBatchJobs(): Promise<BatchJob[]> {
    try {
      return await invoke('get_batch_jobs');
    } catch (error) {
      console.error('Failed to get batch jobs:', error);
      throw new Error(`Failed to get batch jobs: ${error}`);
    }
  }

  async getBatchJob(jobId: string): Promise<BatchJob | null> {
    try {
      return await invoke('get_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to get batch job:', error);
      throw new Error(`Failed to get batch job: ${error}`);
    }
  }

  async createBatchJob(jobData: any): Promise<string> {
    try {
      return await invoke('create_batch_job', { job_data: jobData });
    } catch (error) {
      console.error('Failed to create batch job:', error);
      throw new Error(`Failed to create batch job: ${error}`);
    }
  }

  async startBatchJob(jobId: string): Promise<void> {
    try {
      await invoke('start_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to start batch job:', error);
      throw new Error(`Failed to start batch job: ${error}`);
    }
  }

  async pauseBatchJob(jobId: string): Promise<void> {
    try {
      await invoke('pause_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to pause batch job:', error);
      throw new Error(`Failed to pause batch job: ${error}`);
    }
  }

  async cancelBatchJob(jobId: string): Promise<void> {
    try {
      await invoke('cancel_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to cancel batch job:', error);
      throw new Error(`Failed to cancel batch job: ${error}`);
    }
  }

  async deleteBatchJob(jobId: string): Promise<void> {
    try {
      await invoke('delete_batch_job', { job_id: jobId });
    } catch (error) {
      console.error('Failed to delete batch job:', error);
      throw new Error(`Failed to delete batch job: ${error}`);
    }
  }

  async getBatchJobCount(): Promise<number> {
    try {
      return await invoke('get_batch_job_count');
    } catch (error) {
      console.error('Failed to get batch job count:', error);
      return 0;
    }
  }

  async getActiveBatchJobCount(): Promise<number> {
    try {
      return await invoke('get_active_batch_job_count');
    } catch (error) {
      console.error('Failed to get active batch job count:', error);
      return 0;
    }
  }

  // Search functionality
  async searchAll(query: string, limit?: number): Promise<any> {
    try {
      return await invoke('search_all', { query, limit });
    } catch (error) {
      console.error('Failed to search:', error);
      throw new Error(`Failed to search: ${error}`);
    }
  }

  // Security management
  async createApiKey(request: CreateApiKeyRequest): Promise<CreateApiKeyResponse> {
    try {
      return await invoke('create_api_key', { request });
    } catch (error) {
      console.error('Failed to create API key:', error);
      throw new Error(`Failed to create API key: ${error}`);
    }
  }

  async getApiKeys(): Promise<ApiKey[]> {
    try {
      return await invoke('get_api_keys');
    } catch (error) {
      console.error('Failed to get API keys:', error);
      throw new Error(`Failed to get API keys: ${error}`);
    }
  }

  async revokeApiKey(keyId: string): Promise<void> {
    try {
      await invoke('revoke_api_key', { key_id: keyId });
    } catch (error) {
      console.error('Failed to revoke API key:', error);
      throw new Error(`Failed to revoke API key: ${error}`);
    }
  }

  async deleteApiKey(keyId: string): Promise<void> {
    try {
      await invoke('delete_api_key', { key_id: keyId });
    } catch (error) {
      console.error('Failed to delete API key:', error);
      throw new Error(`Failed to delete API key: ${error}`);
    }
  }

  async validateApiKey(rawKey: string): Promise<ApiKey | null> {
    try {
      return await invoke('validate_api_key', { raw_key: rawKey });
    } catch (error) {
      console.error('Failed to validate API key:', error);
      throw new Error(`Failed to validate API key: ${error}`);
    }
  }

  async getSecurityEvents(limit?: number): Promise<SecurityEvent[]> {
    try {
      return await invoke('get_security_events', { limit });
    } catch (error) {
      console.error('Failed to get security events:', error);
      throw new Error(`Failed to get security events: ${error}`);
    }
  }

  async getSecurityMetrics(): Promise<SecurityMetrics> {
    try {
      return await invoke('get_security_metrics');
    } catch (error) {
      console.error('Failed to get security metrics:', error);
      throw new Error(`Failed to get security metrics: ${error}`);
    }
  }

  async clearSecurityEvents(): Promise<void> {
    try {
      await invoke('clear_security_events');
    } catch (error) {
      console.error('Failed to clear security events:', error);
      throw new Error(`Failed to clear security events: ${error}`);
    }
  }
}

// Singleton instance
export const tauriApi = new TauriApiService();