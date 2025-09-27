import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { tauriApi } from '../lib/tauri-api';
import { ModelInfo, SystemInfo, MetricsSnapshot, InferenceParams, InfernoMetrics, ActiveProcessInfo, AppSettings, Notification, BatchJob, SearchResponse, ApiKey, SecurityEvent, SecurityMetrics, CreateApiKeyRequest } from '../types/inferno';
import { toast } from 'react-hot-toast';

// Models
export function useModels() {
  return useQuery({
    queryKey: ['models'],
    queryFn: () => tauriApi.getModels(),
    refetchInterval: 5000, // Refresh every 5 seconds
  });
}

export function useLoadedModels() {
  return useQuery({
    queryKey: ['loaded-models'],
    queryFn: () => tauriApi.getLoadedModels(),
    refetchInterval: 2000, // Refresh every 2 seconds
  });
}

export function useLoadModel() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ modelName, backendType }: { modelName: string; backendType: string }) =>
      tauriApi.loadModel(modelName, backendType),
    onSuccess: (backendId, { modelName }) => {
      toast.success(`Model "${modelName}" loaded successfully`);
      // Invalidate related queries
      queryClient.invalidateQueries({ queryKey: ['loaded-models'] });
      queryClient.invalidateQueries({ queryKey: ['metrics'] });
    },
    onError: (error: any, { modelName }) => {
      toast.error(`Failed to load model "${modelName}": ${error.message}`);
    },
  });
}

export function useUnloadModel() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (backendId: string) => tauriApi.unloadModel(backendId),
    onSuccess: (_, backendId) => {
      toast.success(`Model unloaded successfully`);
      queryClient.invalidateQueries({ queryKey: ['loaded-models'] });
      queryClient.invalidateQueries({ queryKey: ['metrics'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to unload model: ${error.message}`);
    },
  });
}

// Inference
export function useInference() {
  return useMutation({
    mutationFn: ({ backendId, prompt, params }: {
      backendId: string;
      prompt: string;
      params: InferenceParams;
    }) => tauriApi.infer(backendId, prompt, params),
    onError: (error: any) => {
      toast.error(`Inference failed: ${error.message}`);
    },
  });
}

// System info and metrics
export function useSystemInfo() {
  return useQuery({
    queryKey: ['system-info'],
    queryFn: () => tauriApi.getSystemInfo(),
    refetchInterval: 10000, // Refresh every 10 seconds
  });
}

export function useMetrics() {
  return useQuery({
    queryKey: ['metrics'],
    queryFn: () => tauriApi.getMetrics(),
    refetchInterval: 2000, // Refresh every 2 seconds
  });
}

export function useInfernoMetrics() {
  return useQuery({
    queryKey: ['inferno-metrics'],
    queryFn: () => tauriApi.getInfernoMetrics(),
    refetchInterval: 2000, // Refresh every 2 seconds
  });
}

export function useActiveProcesses() {
  return useQuery({
    queryKey: ['active-processes'],
    queryFn: () => tauriApi.getActiveProcesses(),
    refetchInterval: 3000, // Refresh every 3 seconds
  });
}

// File operations
export function useValidateModel() {
  return useMutation({
    mutationFn: (modelPath: string) => tauriApi.validateModel(modelPath),
  });
}

export function useOpenFileDialog() {
  return useMutation({
    mutationFn: () => tauriApi.openFileDialog(),
    onError: (error: any) => {
      toast.error(`Failed to open file dialog: ${error.message}`);
    },
  });
}

// Combined hook for dashboard data
export function useDashboardData() {
  const models = useModels();
  const loadedModels = useLoadedModels();
  const systemInfo = useSystemInfo();
  const metrics = useMetrics();

  return {
    models,
    loadedModels,
    systemInfo,
    metrics,
    isLoading: models.isLoading || loadedModels.isLoading || systemInfo.isLoading || metrics.isLoading,
    error: models.error || loadedModels.error || systemInfo.error || metrics.error,
  };
}

// Activity logging
export function useRecentActivities(limit?: number) {
  return useQuery({
    queryKey: ['recent-activities', limit],
    queryFn: () => tauriApi.getRecentActivities(limit),
    refetchInterval: 5000, // Refresh every 5 seconds
  });
}

export function useActivityStats() {
  return useQuery({
    queryKey: ['activity-stats'],
    queryFn: () => tauriApi.getActivityStats(),
    refetchInterval: 10000, // Refresh every 10 seconds
  });
}

export function useClearActivities() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => tauriApi.clearActivities(),
    onSuccess: () => {
      toast.success('Activities cleared successfully');
      queryClient.invalidateQueries({ queryKey: ['recent-activities'] });
      queryClient.invalidateQueries({ queryKey: ['activity-stats'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to clear activities: ${error.message}`);
    },
  });
}

// Settings management
export function useSettings() {
  return useQuery({
    queryKey: ['settings'],
    queryFn: () => tauriApi.getSettings(),
    staleTime: 30000, // Settings don't change often, cache for 30 seconds
  });
}

export function useUpdateSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (settings: AppSettings) => tauriApi.setSettings(settings),
    onSuccess: () => {
      toast.success('Settings saved successfully');
      queryClient.invalidateQueries({ queryKey: ['settings'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to save settings: ${error.message}`);
    },
  });
}

// Notification management
export function useNotifications() {
  return useQuery({
    queryKey: ['notifications'],
    queryFn: () => tauriApi.getNotifications(),
    refetchInterval: 10000, // Refresh every 10 seconds
  });
}

export function useUnreadNotificationCount() {
  return useQuery({
    queryKey: ['unread-notification-count'],
    queryFn: () => tauriApi.getUnreadNotificationCount(),
    refetchInterval: 5000, // Refresh every 5 seconds for real-time updates
  });
}

export function useMarkNotificationAsRead() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (notificationId: string) => tauriApi.markNotificationAsRead(notificationId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notifications'] });
      queryClient.invalidateQueries({ queryKey: ['unread-notification-count'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to mark notification as read: ${error.message}`);
    },
  });
}

export function useMarkAllNotificationsAsRead() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => tauriApi.markAllNotificationsAsRead(),
    onSuccess: () => {
      toast.success('All notifications marked as read');
      queryClient.invalidateQueries({ queryKey: ['notifications'] });
      queryClient.invalidateQueries({ queryKey: ['unread-notification-count'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to mark all notifications as read: ${error.message}`);
    },
  });
}

export function useDismissNotification() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (notificationId: string) => tauriApi.dismissNotification(notificationId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notifications'] });
      queryClient.invalidateQueries({ queryKey: ['unread-notification-count'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to dismiss notification: ${error.message}`);
    },
  });
}

export function useClearAllNotifications() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => tauriApi.clearAllNotifications(),
    onSuccess: () => {
      toast.success('All notifications cleared');
      queryClient.invalidateQueries({ queryKey: ['notifications'] });
      queryClient.invalidateQueries({ queryKey: ['unread-notification-count'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to clear all notifications: ${error.message}`);
    },
  });
}

export function useCreateNotification() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (notification: Omit<Notification, 'id' | 'timestamp'>) =>
      tauriApi.createNotification(notification),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notifications'] });
      queryClient.invalidateQueries({ queryKey: ['unread-notification-count'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to create notification: ${error.message}`);
    },
  });
}

// Batch job management
export function useBatchJobs() {
  return useQuery({
    queryKey: ['batch-jobs'],
    queryFn: () => tauriApi.getBatchJobs(),
    refetchInterval: 3000, // Refresh every 3 seconds for job status updates
  });
}

export function useBatchJob(jobId: string) {
  return useQuery({
    queryKey: ['batch-job', jobId],
    queryFn: () => tauriApi.getBatchJob(jobId),
    enabled: !!jobId,
    refetchInterval: 2000, // Refresh every 2 seconds for running jobs
  });
}

export function useCreateBatchJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (jobData: any) => tauriApi.createBatchJob(jobData),
    onSuccess: () => {
      toast.success('Batch job created successfully');
      queryClient.invalidateQueries({ queryKey: ['batch-jobs'] });
      queryClient.invalidateQueries({ queryKey: ['active-processes'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to create batch job: ${error.message}`);
    },
  });
}

export function useStartBatchJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (jobId: string) => tauriApi.startBatchJob(jobId),
    onSuccess: () => {
      toast.success('Batch job started');
      queryClient.invalidateQueries({ queryKey: ['batch-jobs'] });
      queryClient.invalidateQueries({ queryKey: ['active-processes'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to start batch job: ${error.message}`);
    },
  });
}

export function usePauseBatchJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (jobId: string) => tauriApi.pauseBatchJob(jobId),
    onSuccess: () => {
      toast.success('Batch job paused');
      queryClient.invalidateQueries({ queryKey: ['batch-jobs'] });
      queryClient.invalidateQueries({ queryKey: ['active-processes'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to pause batch job: ${error.message}`);
    },
  });
}

export function useCancelBatchJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (jobId: string) => tauriApi.cancelBatchJob(jobId),
    onSuccess: () => {
      toast.success('Batch job cancelled');
      queryClient.invalidateQueries({ queryKey: ['batch-jobs'] });
      queryClient.invalidateQueries({ queryKey: ['active-processes'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to cancel batch job: ${error.message}`);
    },
  });
}

export function useDeleteBatchJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (jobId: string) => tauriApi.deleteBatchJob(jobId),
    onSuccess: () => {
      toast.success('Batch job deleted');
      queryClient.invalidateQueries({ queryKey: ['batch-jobs'] });
      queryClient.invalidateQueries({ queryKey: ['active-processes'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to delete batch job: ${error.message}`);
    },
  });
}

export function useBatchJobCount() {
  return useQuery({
    queryKey: ['batch-job-count'],
    queryFn: () => tauriApi.getBatchJobCount(),
    refetchInterval: 5000, // Refresh every 5 seconds
  });
}

export function useActiveBatchJobCount() {
  return useQuery({
    queryKey: ['active-batch-job-count'],
    queryFn: () => tauriApi.getActiveBatchJobCount(),
    refetchInterval: 3000, // Refresh every 3 seconds
  });
}

// Search functionality
export function useSearch(query: string, enabled: boolean = true) {
  return useQuery({
    queryKey: ['search', query],
    queryFn: () => tauriApi.searchAll(query, 20),
    enabled: enabled && query.trim().length > 0,
    staleTime: 30000, // Keep results fresh for 30 seconds
    refetchOnWindowFocus: false,
  });
}

// Security management
export function useApiKeys() {
  return useQuery({
    queryKey: ['api-keys'],
    queryFn: () => tauriApi.getApiKeys(),
    refetchInterval: 30000, // Refresh every 30 seconds
  });
}

export function useCreateApiKey() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CreateApiKeyRequest) => tauriApi.createApiKey(request),
    onSuccess: () => {
      toast.success('API key created successfully');
      queryClient.invalidateQueries({ queryKey: ['api-keys'] });
      queryClient.invalidateQueries({ queryKey: ['security-metrics'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to create API key: ${error.message}`);
    },
  });
}

export function useRevokeApiKey() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (keyId: string) => tauriApi.revokeApiKey(keyId),
    onSuccess: () => {
      toast.success('API key revoked successfully');
      queryClient.invalidateQueries({ queryKey: ['api-keys'] });
      queryClient.invalidateQueries({ queryKey: ['security-metrics'] });
      queryClient.invalidateQueries({ queryKey: ['security-events'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to revoke API key: ${error.message}`);
    },
  });
}

export function useDeleteApiKey() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (keyId: string) => tauriApi.deleteApiKey(keyId),
    onSuccess: () => {
      toast.success('API key deleted successfully');
      queryClient.invalidateQueries({ queryKey: ['api-keys'] });
      queryClient.invalidateQueries({ queryKey: ['security-metrics'] });
      queryClient.invalidateQueries({ queryKey: ['security-events'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to delete API key: ${error.message}`);
    },
  });
}

export function useValidateApiKey() {
  return useMutation({
    mutationFn: (rawKey: string) => tauriApi.validateApiKey(rawKey),
    onError: (error: any) => {
      toast.error(`Failed to validate API key: ${error.message}`);
    },
  });
}

export function useSecurityEvents(limit?: number) {
  return useQuery({
    queryKey: ['security-events', limit],
    queryFn: () => tauriApi.getSecurityEvents(limit),
    refetchInterval: 10000, // Refresh every 10 seconds
  });
}

export function useSecurityMetrics() {
  return useQuery({
    queryKey: ['security-metrics'],
    queryFn: () => tauriApi.getSecurityMetrics(),
    refetchInterval: 30000, // Refresh every 30 seconds
  });
}

export function useClearSecurityEvents() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => tauriApi.clearSecurityEvents(),
    onSuccess: () => {
      toast.success('Security events cleared successfully');
      queryClient.invalidateQueries({ queryKey: ['security-events'] });
    },
    onError: (error: any) => {
      toast.error(`Failed to clear security events: ${error.message}`);
    },
  });
}