// Core Inferno API Types
export interface ModelInfo {
  id: string;
  name: string;
  path: string;
  format: 'gguf' | 'onnx' | 'pytorch' | 'safetensors';
  backend_type?: 'gguf' | 'onnx' | 'pytorch';
  size: number;
  checksum: string;
  metadata?: {
    architecture?: string;
    parameters?: number;
    quantization?: string;
    context_length?: number;
    created_at?: string;
    description?: string;
  };
  status: 'available' | 'loading' | 'loaded' | 'error';
}

export interface InferenceParams {
  temperature?: number;
  top_k?: number;
  top_p?: number;
  max_tokens?: number;
  stream?: boolean;
  stop_sequences?: string[];
  seed?: number;
  context_length?: number;
  batch_size?: number;
}

export interface InferenceRequest {
  model_id: string;
  prompt: string;
  params?: InferenceParams;
}

export interface InferenceResponse {
  id: string;
  model_id: string;
  prompt: string;
  response: string;
  tokens_generated: number;
  inference_time_ms: number;
  tokens_per_second: number;
  timestamp: string;
  status: 'success' | 'error' | 'pending';
  error?: string;
}

export interface SystemMetrics {
  timestamp: string;
  cpu_usage: number;
  memory_usage: number;
  memory_total: number;
  gpu_usage?: number;
  gpu_memory_usage?: number;
  gpu_memory_total?: number;
  active_models: number;
  total_inferences: number;
  avg_inference_time: number;
  errors_count: number;
  uptime_seconds: number;
}

export interface ModelMetrics {
  model_id: string;
  inference_count: number;
  avg_inference_time: number;
  total_tokens: number;
  error_count: number;
  last_used: string;
  cache_hits: number;
  cache_misses: number;
}

export interface BatchJob {
  id: string;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  model_id: string;
  created_at: string;
  started_at?: string;
  completed_at?: string;
  progress: number;
  total_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  schedule?: string;
  next_run?: string;
  config: {
    inputs: string[];
    output_format: string;
    batch_size: number;
    parallel_workers: number;
  };
  results?: {
    outputs: string[];
    errors: string[];
    metrics: {
      total_time: number;
      avg_time_per_task: number;
      throughput: number;
    };
  };
}

export interface SecurityEvent {
  id: string;
  event_type: SecurityEventType;
  severity: SecuritySeverity;
  timestamp: string;
  source_ip?: string;
  user_agent?: string;
  api_key_id?: string;
  description: string;
  metadata: Record<string, any>;
}

export type SecurityEventType =
  | 'apikeyCreated'
  | 'apikeyRevoked'
  | 'apikeyUsed'
  | 'unauthorizedAccess'
  | 'authenticationFailed'
  | 'permissionDenied'
  | 'suspiciousActivity'
  | 'configurationChanged';

export type SecuritySeverity = 'low' | 'medium' | 'high' | 'critical';

export interface AuditLog {
  id: string;
  timestamp: string;
  user_id?: string;
  action: string;
  resource_type: string;
  resource_id?: string;
  details: Record<string, any>;
  ip_address?: string;
  user_agent?: string;
  status: 'success' | 'failure';
}

export interface ConversionJob {
  id: string;
  source_path: string;
  target_path: string;
  source_format: string;
  target_format: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress: number;
  created_at: string;
  started_at?: string;
  completed_at?: string;
  options: {
    quantization?: string;
    optimization_level?: 'fast' | 'balanced' | 'aggressive';
    precision?: 'fp16' | 'fp32' | 'int8';
    batch_size?: number;
  };
  error?: string;
}

export interface Tenant {
  id: string;
  name: string;
  status: 'active' | 'suspended' | 'inactive';
  created_at: string;
  limits: {
    max_models: number;
    max_concurrent_inferences: number;
    max_batch_jobs: number;
    storage_quota_gb: number;
  };
  usage: {
    models_count: number;
    storage_used_gb: number;
    monthly_inferences: number;
    monthly_compute_hours: number;
  };
  settings: {
    auto_scaling: boolean;
    monitoring_enabled: boolean;
    audit_retention_days: number;
  };
}

export interface ApiKey {
  id: string;
  name: string;
  key_hash: string;
  key_prefix: string;
  permissions: string[];
  created_at: string;
  last_used?: string;
  expires_at?: string;
  is_active: boolean;
  usage_count: number;
  created_by: string;
}

export interface WebSocketMessage {
  type: 'metrics' | 'inference_stream' | 'job_update' | 'security_alert' | 'system_status';
  data: any;
  timestamp: string;
}

export interface DashboardConfig {
  refresh_interval: number;
  max_log_entries: number;
  default_chart_range: '1h' | '6h' | '24h' | '7d' | '30d';
  theme: 'light' | 'dark' | 'auto';
  notifications: {
    browser_notifications: boolean;
    email_alerts: boolean;
    slack_webhook?: string;
  };
  widgets: {
    id: string;
    type: string;
    position: { x: number; y: number };
    size: { width: number; height: number };
    config: Record<string, any>;
  }[];
}

// Additional types for Tauri API
export interface SystemInfo {
  cpu_name: string;
  cpu_usage: number;
  cpu_cores: number;
  cpu_frequency?: string;
  total_memory: number;
  used_memory: number;
  available_memory: number;
  platform: string;
  arch: string;
}

export interface MetricsSnapshot {
  inference_count: number;
  success_count: number;
  error_count: number;
  average_latency: number;
  models_loaded: number;
  active_streaming_sessions?: number;
  active_models?: number;
  cpu_usage?: number;
  memory_usage?: number;
}

export interface InfernoMetrics {
  cpu_usage: number;
  memory_usage: number;
  gpu_usage?: number;
  active_models: number;
  models_loaded?: number;
  active_inferences: number;
  active_streaming_sessions: number;
  inference_count: number;
  success_count: number;
  error_count: number;
  average_latency: number;
  avg_response_time_ms?: number;
}

export interface ActiveProcessInfo {
  active_models: string[];
  active_inferences: number;
  batch_jobs: number;
  streaming_sessions: number;
}

export interface Notification {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  timestamp: string;
  read: boolean;
  action?: {
    label: string;
    url?: string;
    callback?: string;
  };
  source: 'system' | 'inference' | 'security' | 'batch' | 'model';
  priority: 'low' | 'medium' | 'high' | 'critical';
  metadata?: Record<string, any>;
}

export interface NativeNotificationPayload {
  title: string;
  body: string;
  icon?: string;
  sound?: string;
}

export interface NotificationSettings {
  enabled: boolean;
  types: {
    system: boolean;
    inference: boolean;
    security: boolean;
    batch: boolean;
    model: boolean;
  };
  priority_filter: 'all' | 'medium' | 'high' | 'critical';
  auto_dismiss_after: number; // seconds, 0 = no auto dismiss
  max_notifications: number;
}

export interface AppSettings {
  // Model Settings
  modelsDirectory: string;
  autoDiscoverModels: boolean;

  // Inference Settings
  defaultTemperature: number;
  defaultMaxTokens: number;
  defaultTopP: number;
  defaultTopK: number;

  // System Settings
  maxMemoryUsage: number;
  preferGPU: boolean;
  maxConcurrentInferences: number;

  // Cache Settings
  enableCache: boolean;
  cacheDirectory: string;
  maxCacheSize: number;

  // API Settings
  enableRestAPI: boolean;
  apiPort: number;
  enableCORS: boolean;

  // Security Settings
  requireAuthentication: boolean;
  enableAuditLog: boolean;
  logLevel: 'error' | 'warn' | 'info' | 'debug';

  // Notification Settings
  notifications: NotificationSettings;
}

// Search Types
export interface SearchResult {
  id: string;
  type: 'model' | 'batch_job' | 'notification' | 'setting' | 'page';
  title: string;
  description?: string;
  url?: string;
  metadata?: Record<string, any>;
  relevance_score: number;
}

export interface SearchResponse {
  results: SearchResult[];
  total_count: number;
  query: string;
  search_time_ms: number;
}

// Additional Security Types
export interface SecurityMetrics {
  total_api_keys: number;
  active_api_keys: number;
  expired_api_keys: number;
  security_events_24h: number;
  failed_auth_attempts_24h: number;
  suspicious_activities_24h: number;
  last_security_scan?: string;
}

export interface CreateApiKeyRequest {
  name: string;
  permissions: string[];
  expires_in_days?: number;
}

export interface CreateApiKeyResponse {
  api_key: ApiKey;
  raw_key: string;
}
