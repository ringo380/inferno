// Core Inferno API Types
export interface ModelInfo {
  id: string;
  name: string;
  path: string;
  format: 'gguf' | 'onnx' | 'pytorch' | 'safetensors';
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
  timestamp: string;
  event_type: 'auth_success' | 'auth_failure' | 'rate_limit' | 'suspicious_activity';
  user_id?: string;
  ip_address: string;
  user_agent?: string;
  details: Record<string, any>;
  severity: 'low' | 'medium' | 'high' | 'critical';
}

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
  key_prefix: string;
  tenant_id?: string;
  permissions: string[];
  expires_at?: string;
  created_at: string;
  last_used?: string;
  usage_count: number;
  status: 'active' | 'revoked' | 'expired';
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