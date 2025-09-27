'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { Progress } from '@/components/ui/progress';
import { useSystemInfo, useRecentActivities, useSettings } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Activity,
  Eye,
  Search,
  Filter,
  Download,
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  Clock,
  Database,
  Server,
  Zap,
  Bell,
  Settings,
  BarChart3,
  TrendingUp,
  FileText,
  Terminal,
  Bug,
} from 'lucide-react';
import { useState } from 'react';

// Mock log data for demonstration
const mockLogs = [
  {
    id: 'log_1',
    timestamp: '2024-01-20T14:22:15.123Z',
    level: 'INFO',
    service: 'inference-engine',
    message: 'Model llama-7b loaded successfully',
    metadata: { model_id: 'llama-7b', load_time_ms: 2341 },
    trace_id: 'trace_abc123',
    request_id: 'req_xyz789'
  },
  {
    id: 'log_2',
    timestamp: '2024-01-20T14:22:10.567Z',
    level: 'WARN',
    service: 'api-gateway',
    message: 'Rate limit exceeded for API key',
    metadata: { api_key: 'key_prod_***', limit: 100, window: '1h' },
    trace_id: 'trace_def456',
    request_id: 'req_abc123'
  },
  {
    id: 'log_3',
    timestamp: '2024-01-20T14:22:05.890Z',
    level: 'ERROR',
    service: 'model-loader',
    message: 'Failed to load model: insufficient memory',
    metadata: { model_path: '/models/large-model.gguf', required_memory: '16GB', available_memory: '12GB' },
    trace_id: 'trace_ghi789',
    request_id: 'req_def456'
  },
  {
    id: 'log_4',
    timestamp: '2024-01-20T14:21:58.234Z',
    level: 'DEBUG',
    service: 'batch-processor',
    message: 'Processing batch job item 45/100',
    metadata: { job_id: 'batch_001', progress: 45, total: 100 },
    trace_id: 'trace_jkl012',
    request_id: 'req_ghi789'
  },
  {
    id: 'log_5',
    timestamp: '2024-01-20T14:21:45.678Z',
    level: 'INFO',
    service: 'health-check',
    message: 'System health check completed',
    metadata: { status: 'healthy', checks: ['cpu', 'memory', 'disk', 'network'] },
    trace_id: 'trace_mno345',
    request_id: 'req_jkl012'
  }
];

// Mock alerts configuration
const mockAlerts = [
  {
    id: 'alert_1',
    name: 'High CPU Usage',
    condition: 'cpu_usage > 90%',
    severity: 'critical',
    enabled: true,
    last_triggered: '2024-01-19T16:30:00Z',
    trigger_count: 3
  },
  {
    id: 'alert_2',
    name: 'Model Load Failure',
    condition: 'log.level = ERROR AND log.service = model-loader',
    severity: 'high',
    enabled: true,
    last_triggered: '2024-01-20T14:22:05Z',
    trigger_count: 1
  },
  {
    id: 'alert_3',
    name: 'API Rate Limiting',
    condition: 'rate_limit_exceeded',
    severity: 'medium',
    enabled: true,
    last_triggered: '2024-01-20T14:22:10Z',
    trigger_count: 7
  },
  {
    id: 'alert_4',
    name: 'Low Disk Space',
    condition: 'disk_usage > 85%',
    severity: 'medium',
    enabled: false,
    last_triggered: null,
    trigger_count: 0
  }
];

// Mock metrics data
const mockMetrics = {
  logs_per_minute: 142,
  error_rate: 2.3,
  active_traces: 23,
  storage_usage: 67,
  retention_days: 30
};

function formatTimestamp(timestamp: string): string {
  return new Date(timestamp).toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  });
}

function formatRelativeTime(timestamp: string): string {
  const now = new Date();
  const time = new Date(timestamp);
  const diffMs = now.getTime() - time.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  return `${diffDays}d ago`;
}

function getLogLevelColor(level: string): string {
  switch (level.toUpperCase()) {
    case 'ERROR':
      return 'text-red-600 bg-red-50 border-red-200';
    case 'WARN':
      return 'text-yellow-600 bg-yellow-50 border-yellow-200';
    case 'INFO':
      return 'text-blue-600 bg-blue-50 border-blue-200';
    case 'DEBUG':
      return 'text-gray-600 bg-gray-50 border-gray-200';
    default:
      return 'text-gray-600 bg-gray-50 border-gray-200';
  }
}

function getLogLevelIcon(level: string) {
  switch (level.toUpperCase()) {
    case 'ERROR':
      return AlertTriangle;
    case 'WARN':
      return AlertTriangle;
    case 'INFO':
      return CheckCircle;
    case 'DEBUG':
      return Bug;
    default:
      return FileText;
  }
}

function getSeverityColor(severity: string): string {
  switch (severity) {
    case 'critical':
      return 'text-red-600 bg-red-50 border-red-200';
    case 'high':
      return 'text-orange-600 bg-orange-50 border-orange-200';
    case 'medium':
      return 'text-yellow-600 bg-yellow-50 border-yellow-200';
    case 'low':
      return 'text-blue-600 bg-blue-50 border-blue-200';
    default:
      return 'text-gray-600 bg-gray-50 border-gray-200';
  }
}

export default function ObservabilityPage() {
  const [logFilter, setLogFilter] = useState('ALL');
  const [showMetadata, setShowMetadata] = useState(false);
  const { data: systemInfo, isLoading: systemLoading } = useSystemInfo();
  const { data: activities, isLoading: activitiesLoading } = useRecentActivities(20);
  const { data: settings, isLoading: settingsLoading } = useSettings();

  const handleExportLogs = () => {
    const data = {
      timestamp: new Date().toISOString(),
      filter: logFilter,
      logs: mockLogs,
      summary: {
        total_logs: mockLogs.length,
        error_count: mockLogs.filter(log => log.level === 'ERROR').length,
        warn_count: mockLogs.filter(log => log.level === 'WARN').length,
        info_count: mockLogs.filter(log => log.level === 'INFO').length,
        debug_count: mockLogs.filter(log => log.level === 'DEBUG').length,
      }
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `inferno-logs-${Date.now()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const filteredLogs = logFilter === 'ALL'
    ? mockLogs
    : mockLogs.filter(log => log.level === logFilter);

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Observability</h1>
          <p className="text-muted-foreground">
            Monitor system logs, traces, and alerts
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant="outline" className="flex items-center gap-1">
            <Activity className="h-3 w-3" />
            {mockMetrics.logs_per_minute} logs/min
          </Badge>
          <Button variant="outline" onClick={handleExportLogs}>
            <Download className="h-4 w-4 mr-2" />
            Export Logs
          </Button>
        </div>
      </div>

      {/* Observability Overview */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Log Volume</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockMetrics.logs_per_minute}</div>
            <p className="text-xs text-muted-foreground">
              logs per minute
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Error Rate</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">{mockMetrics.error_rate}%</div>
            <p className="text-xs text-muted-foreground">
              of all requests
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Traces</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockMetrics.active_traces}</div>
            <p className="text-xs text-muted-foreground">
              distributed traces
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Storage Usage</CardTitle>
            <Database className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockMetrics.storage_usage}%</div>
            <p className="text-xs text-muted-foreground">
              {mockMetrics.retention_days} day retention
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Alerts Configuration */}
      <Card>
        <CardHeader>
          <CardTitle>Alert Rules</CardTitle>
          <CardDescription>Configure monitoring alerts and thresholds</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {mockAlerts.map((alert) => (
              <div key={alert.id} className={`p-4 rounded-lg border ${getSeverityColor(alert.severity)}`}>
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3">
                      <div>
                        <div className="font-medium">{alert.name}</div>
                        <div className="text-sm text-muted-foreground">
                          {alert.condition}
                        </div>
                      </div>
                      <Badge variant="outline" className="text-xs">
                        {alert.severity}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-4 mt-2 text-sm text-muted-foreground">
                      <span>Triggered {alert.trigger_count} times</span>
                      {alert.last_triggered && (
                        <span>Last: {formatRelativeTime(alert.last_triggered)}</span>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Switch checked={alert.enabled} />
                    <Button variant="ghost" size="icon">
                      <Settings className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Logs Viewer */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>System Logs</CardTitle>
              <CardDescription>Real-time log streaming and analysis</CardDescription>
            </div>
            <div className="flex items-center gap-2">
              <div className="flex items-center gap-2">
                <Label htmlFor="metadata-toggle" className="text-sm">Show metadata</Label>
                <Switch
                  id="metadata-toggle"
                  checked={showMetadata}
                  onCheckedChange={setShowMetadata}
                />
              </div>
              <Button variant="outline" size="sm">
                <Filter className="h-4 w-4 mr-2" />
                Filter
              </Button>
              <Button variant="outline" size="sm">
                <RefreshCw className="h-4 w-4 mr-2" />
                Refresh
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {/* Log Level Filter */}
          <div className="flex gap-2 mb-4">
            {['ALL', 'ERROR', 'WARN', 'INFO', 'DEBUG'].map((level) => (
              <Button
                key={level}
                variant={logFilter === level ? 'default' : 'outline'}
                size="sm"
                onClick={() => setLogFilter(level)}
              >
                {level}
                {level !== 'ALL' && (
                  <Badge variant="secondary" className="ml-2 text-xs">
                    {mockLogs.filter(log => log.level === level).length}
                  </Badge>
                )}
              </Button>
            ))}
          </div>

          {/* Logs List */}
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {filteredLogs.map((log) => {
              const LogIcon = getLogLevelIcon(log.level);
              const levelClass = getLogLevelColor(log.level);

              return (
                <div key={log.id} className="font-mono text-sm border rounded p-3 bg-background">
                  <div className="flex items-start gap-3">
                    <div className={`p-1 rounded border ${levelClass}`}>
                      <LogIcon className="h-3 w-3" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-3 mb-1">
                        <span className="text-muted-foreground text-xs">
                          {formatTimestamp(log.timestamp)}
                        </span>
                        <Badge variant="outline" className="text-xs">
                          {log.level}
                        </Badge>
                        <Badge variant="secondary" className="text-xs">
                          {log.service}
                        </Badge>
                        {log.trace_id && (
                          <code className="text-xs bg-muted px-1 rounded">
                            {log.trace_id}
                          </code>
                        )}
                      </div>
                      <div className="text-foreground mb-2">
                        {log.message}
                      </div>
                      {showMetadata && log.metadata && (
                        <div className="text-xs text-muted-foreground bg-muted p-2 rounded">
                          <pre>{JSON.stringify(log.metadata, null, 2)}</pre>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>

          {filteredLogs.length === 0 && (
            <div className="text-center py-8 text-muted-foreground">
              <Terminal className="h-8 w-8 mx-auto mb-2" />
              <p>No logs found for the selected filter</p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Storage and Retention */}
      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Storage Management</CardTitle>
            <CardDescription>Log storage usage and retention policies</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Storage Usage</span>
                  <span className="text-sm">{mockMetrics.storage_usage}%</span>
                </div>
                <Progress value={mockMetrics.storage_usage} className="h-2" />
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span>Retention Period</span>
                  <span>{mockMetrics.retention_days} days</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span>Compression</span>
                  <Badge variant="secondary">Enabled</Badge>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span>Auto-cleanup</span>
                  <Badge variant="secondary">Enabled</Badge>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Health Monitoring</CardTitle>
            <CardDescription>System health checks and status</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {[
                { name: 'Log Ingestion', status: 'healthy', latency: '12ms' },
                { name: 'Search Index', status: 'healthy', latency: '8ms' },
                { name: 'Alert Engine', status: 'healthy', latency: '5ms' },
                { name: 'Storage Backend', status: 'warning', latency: '45ms' }
              ].map((check) => (
                <div key={check.name} className="flex items-center justify-between p-2 border rounded">
                  <div className="flex items-center gap-2">
                    {check.status === 'healthy' ? (
                      <CheckCircle className="h-4 w-4 text-green-600" />
                    ) : (
                      <AlertTriangle className="h-4 w-4 text-yellow-600" />
                    )}
                    <span className="text-sm font-medium">{check.name}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-muted-foreground">{check.latency}</span>
                    <Badge variant={check.status === 'healthy' ? 'default' : 'secondary'}>
                      {check.status}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}