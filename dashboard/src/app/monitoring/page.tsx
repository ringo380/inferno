'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { MetricCard } from '@/components/dashboard/metric-card';
import { SystemChart } from '@/components/dashboard/system-chart';
import { useSystemInfo, useMetrics, useRecentActivities } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';
import { AnimatedCounter } from '@/components/ui/animated-counter';
import {
  Activity,
  Cpu,
  HardDrive,
  Zap,
  Brain,
  Clock,
  TrendingUp,
  BarChart3,
  RefreshCw,
  Download,
  AlertTriangle,
  CheckCircle,
  Server,
  Database,
} from 'lucide-react';

function formatMemorySize(bytes: number | undefined): string {
  const gb = (bytes || 0) / (1024 * 1024 * 1024);
  return `${gb.toFixed(1)} GB`;
}

function formatPercentage(value: number | undefined): string {
  return `${(value || 0).toFixed(1)}%`;
}

export default function MonitoringPage() {
  const { data: systemInfo, isLoading: systemLoading, error: systemError, refetch: refetchSystem } = useSystemInfo();
  const { data: metrics, isLoading: metricsLoading, error: metricsError, refetch: refetchMetrics } = useMetrics();
  const { data: activities, isLoading: activitiesLoading } = useRecentActivities(20);

  const handleRefresh = () => {
    refetchSystem();
    refetchMetrics();
  };

  const handleExportMetrics = () => {
    const data = {
      timestamp: new Date().toISOString(),
      systemInfo,
      metrics,
      activities,
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `inferno-metrics-${Date.now()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  if (systemError || metricsError) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">System Monitoring</h1>
            <p className="text-muted-foreground">Real-time system performance and metrics</p>
          </div>
        </div>

        <Card className="border-destructive">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertTriangle className="h-5 w-5" />
              Error Loading Monitoring Data
            </CardTitle>
            <CardDescription>
              Failed to connect to system monitoring. Please ensure the application is running properly.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              {systemError?.message || metricsError?.message || 'Unknown error occurred'}
            </p>
            <Button onClick={handleRefresh} className="mt-4">
              <RefreshCw className="h-4 w-4 mr-2" />
              Retry
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">System Monitoring</h1>
          <p className="text-muted-foreground">
            Real-time system performance and metrics
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant={systemInfo ? "success" : "secondary"} className="flex items-center gap-1">
            <Activity className="h-3 w-3" />
            {systemInfo ? "Monitoring Active" : "Connecting..."}
          </Badge>
          <Button variant="outline" onClick={handleRefresh}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
          <Button onClick={handleExportMetrics}>
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
        </div>
      </div>

      {/* System Overview Metrics */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {systemLoading || metricsLoading ? (
          <>
            {[...Array(4)].map((_, i) => (
              <Card key={i}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <Skeleton className="h-4 w-24" />
                  <Skeleton className="h-4 w-4" />
                </CardHeader>
                <CardContent>
                  <Skeleton className="h-8 w-16 mb-2" />
                  <Skeleton className="h-3 w-32" />
                </CardContent>
              </Card>
            ))}
          </>
        ) : (
          <>
            <MetricCard
              title="CPU Usage"
              value={<AnimatedCounter value={systemInfo?.cpu_usage || 0} suffix="%" />}
              description={`${systemInfo?.cpu_cores || 0} cores available`}
              icon={Cpu}
              trend="stable"
              color="blue"
            />
            <MetricCard
              title="Memory Usage"
              value={formatMemorySize(systemInfo?.used_memory)}
              description={`${formatPercentage((systemInfo?.used_memory || 0) / (systemInfo?.total_memory || 1) * 100)} of ${formatMemorySize(systemInfo?.total_memory)}`}
              icon={HardDrive}
              trend="stable"
              color="green"
            />
            <MetricCard
              title="Active Models"
              value={<AnimatedCounter value={metrics?.models_loaded || 0} />}
              description={`${metrics?.inference_count || 0} total inferences`}
              icon={Brain}
              trend="stable"
              color="yellow"
            />
            <MetricCard
              title="Avg Latency"
              value={<AnimatedCounter value={Math.round(metrics?.average_latency || 0)} suffix="ms" />}
              description={`${formatPercentage((metrics?.success_count || 0) / Math.max(metrics?.inference_count || 1, 1) * 100)} success rate`}
              icon={Zap}
              trend="stable"
              color="red"
            />
          </>
        )}
      </div>

      {/* System Performance Chart */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BarChart3 className="h-5 w-5" />
            System Performance
          </CardTitle>
          <CardDescription>
            Real-time system resource usage and inference performance
          </CardDescription>
        </CardHeader>
        <CardContent>
          <SystemChart />
        </CardContent>
      </Card>

      {/* Detailed System Information */}
      <div className="grid gap-6 md:grid-cols-2">
        {/* System Details */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Server className="h-5 w-5" />
              System Details
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {systemLoading ? (
              <div className="space-y-3">
                {[...Array(6)].map((_, i) => (
                  <div key={i} className="flex justify-between">
                    <Skeleton className="h-4 w-24" />
                    <Skeleton className="h-4 w-32" />
                  </div>
                ))}
              </div>
            ) : (
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Platform:</span>
                  <span className="font-medium">{systemInfo?.platform || 'Unknown'}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Architecture:</span>
                  <span className="font-medium">{systemInfo?.arch || 'Unknown'}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">CPU Cores:</span>
                  <span className="font-medium">{systemInfo?.cpu_cores || 0}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Total Memory:</span>
                  <span className="font-medium">{formatMemorySize(systemInfo?.total_memory)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Available Memory:</span>
                  <span className="font-medium">{formatMemorySize(systemInfo?.available_memory)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Memory Usage:</span>
                  <div className="flex items-center space-x-2">
                    <Progress
                      value={(systemInfo?.used_memory || 0) / (systemInfo?.total_memory || 1) * 100}
                      className="w-20 h-2"
                    />
                    <span className="text-sm font-medium">
                      {formatPercentage((systemInfo?.used_memory || 0) / (systemInfo?.total_memory || 1) * 100)}
                    </span>
                  </div>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Performance Metrics */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5" />
              Performance Metrics
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {metricsLoading ? (
              <div className="space-y-3">
                {[...Array(5)].map((_, i) => (
                  <div key={i} className="flex justify-between">
                    <Skeleton className="h-4 w-32" />
                    <Skeleton className="h-4 w-16" />
                  </div>
                ))}
              </div>
            ) : (
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Total Inferences:</span>
                  <span className="font-medium">{metrics?.inference_count || 0}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Successful:</span>
                  <span className="font-medium text-green-600">{metrics?.success_count || 0}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Failed:</span>
                  <span className="font-medium text-red-600">{metrics?.error_count || 0}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Average Latency:</span>
                  <span className="font-medium">{Math.round(metrics?.average_latency || 0)}ms</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Models Loaded:</span>
                  <span className="font-medium">{metrics?.models_loaded || 0}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Success Rate:</span>
                  <div className="flex items-center space-x-2">
                    <Progress
                      value={(metrics?.success_count || 0) / Math.max(metrics?.inference_count || 1, 1) * 100}
                      className="w-20 h-2"
                    />
                    <span className="text-sm font-medium">
                      {formatPercentage((metrics?.success_count || 0) / Math.max(metrics?.inference_count || 1, 1) * 100)}
                    </span>
                  </div>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Recent Activities */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Recent System Activities
          </CardTitle>
          <CardDescription>
            Latest system operations and events
          </CardDescription>
        </CardHeader>
        <CardContent>
          {activitiesLoading ? (
            <div className="space-y-3">
              {[...Array(5)].map((_, i) => (
                <div key={i} className="flex items-start space-x-3">
                  <Skeleton className="h-6 w-6 rounded-full" />
                  <div className="flex-1 space-y-2">
                    <Skeleton className="h-4 w-48" />
                    <Skeleton className="h-3 w-32" />
                  </div>
                </div>
              ))}
            </div>
          ) : activities && activities.length > 0 ? (
            <div className="space-y-3 max-h-[400px] overflow-y-auto">
              {activities.map((activity, index) => (
                <div key={activity.id || index} className="flex items-start space-x-3 p-3 border rounded-lg">
                  <div className={`p-1 rounded-full ${
                    activity.status === 'success' ? 'bg-green-100 text-green-600' :
                    activity.status === 'error' ? 'bg-red-100 text-red-600' :
                    'bg-blue-100 text-blue-600'
                  }`}>
                    {activity.status === 'success' ? (
                      <CheckCircle className="h-3 w-3" />
                    ) : activity.status === 'error' ? (
                      <AlertTriangle className="h-3 w-3" />
                    ) : (
                      <Activity className="h-3 w-3" />
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium">{activity.title}</p>
                      <p className="text-xs text-muted-foreground">
                        {new Date(activity.timestamp).toLocaleTimeString()}
                      </p>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">{activity.description}</p>
                    <div className="flex items-center space-x-2 mt-1">
                      <Badge variant="outline" className="text-xs">
                        {activity.activity_type}
                      </Badge>
                      {activity.user && (
                        <span className="text-xs text-muted-foreground">by {activity.user}</span>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              <Database className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p className="text-sm">No recent activities</p>
              <p className="text-xs">System activities will appear here as they occur</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}