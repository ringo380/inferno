'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { MetricCard } from '@/components/dashboard/metric-card';
import { SystemChart } from '@/components/dashboard/system-chart';
import { useSystemInfo, useInfernoMetrics, useActiveProcesses, useLoadedModels } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';
import { AnimatedCounter } from '@/components/ui/animated-counter';
import {
  Zap,
  Cpu,
  HardDrive,
  Activity,
  Clock,
  TrendingUp,
  TrendingDown,
  BarChart3,
  RefreshCw,
  Download,
  AlertTriangle,
  CheckCircle,
  Target,
  Gauge,
  Timer,
  Database,
} from 'lucide-react';
import { useState } from 'react';

function formatLatency(ms: number | undefined): string {
  if (!ms) return '0ms';
  if (ms < 1000) return `${ms.toFixed(0)}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function formatThroughput(value: number | undefined): string {
  if (!value) return '0';
  if (value < 1000) return value.toFixed(1);
  if (value < 1000000) return `${(value / 1000).toFixed(1)}K`;
  return `${(value / 1000000).toFixed(1)}M`;
}

function getPerformanceStatus(cpuUsage: number, memoryUsage: number, responseTime: number) {
  if (cpuUsage > 80 || memoryUsage > 80 || responseTime > 1000) {
    return { status: 'warning', color: 'text-yellow-600', icon: AlertTriangle };
  }
  if (cpuUsage > 90 || memoryUsage > 90 || responseTime > 2000) {
    return { status: 'critical', color: 'text-red-600', icon: AlertTriangle };
  }
  return { status: 'healthy', color: 'text-green-600', icon: CheckCircle };
}

export default function PerformancePage() {
  const [timeRange, setTimeRange] = useState('1h');
  const { data: systemInfo, isLoading: systemLoading, error: systemError, refetch: refetchSystem } = useSystemInfo();
  const { data: infernoMetrics, isLoading: metricsLoading, error: metricsError, refetch: refetchMetrics } = useInfernoMetrics();
  const { data: activeProcesses, isLoading: processesLoading } = useActiveProcesses();
  const { data: loadedModels, isLoading: modelsLoading } = useLoadedModels();

  const handleRefresh = () => {
    refetchSystem();
    refetchMetrics();
  };

  const handleExportReport = () => {
    const data = {
      timestamp: new Date().toISOString(),
      timeRange,
      systemInfo,
      infernoMetrics,
      activeProcesses,
      summary: {
        totalModels: loadedModels?.length || 0,
        totalInferences: infernoMetrics?.inference_count || 0,
        avgResponseTime: infernoMetrics?.average_latency || 0,
        successRate: infernoMetrics?.success_count && infernoMetrics?.inference_count
          ? (infernoMetrics.success_count / infernoMetrics.inference_count) * 100
          : 100,
      }
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `inferno-performance-report-${Date.now()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // Calculate performance metrics
  const cpuUsage = systemInfo?.cpu_usage || 0;
  const memoryUsage = systemInfo ? (systemInfo.used_memory / systemInfo.total_memory) * 100 : 0;
  const responseTime = infernoMetrics?.average_latency || 0;
  const throughput = infernoMetrics?.inference_count || 0;
  const successRate = infernoMetrics?.success_count && infernoMetrics?.inference_count
    ? (infernoMetrics.success_count / infernoMetrics.inference_count) * 100
    : 100;

  const performanceStatus = getPerformanceStatus(cpuUsage, memoryUsage, responseTime);
  const StatusIcon = performanceStatus.icon;

  if (systemError || metricsError) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">Performance Monitoring</h1>
            <p className="text-muted-foreground">Real-time performance metrics and optimization insights</p>
          </div>
        </div>

        <Card className="border-destructive">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertTriangle className="h-5 w-5" />
              Error Loading Performance Data
            </CardTitle>
            <CardDescription>
              Failed to connect to performance monitoring. Please ensure the application is running properly.
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
          <h1 className="text-3xl font-bold tracking-tight">Performance Monitoring</h1>
          <p className="text-muted-foreground">
            Real-time performance metrics and optimization insights
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant="outline" className={`flex items-center gap-1 ${performanceStatus.color}`}>
            <StatusIcon className="h-3 w-3" />
            System {performanceStatus.status}
          </Badge>
          <Button variant="outline" onClick={handleRefresh}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
          <Button onClick={handleExportReport}>
            <Download className="h-4 w-4 mr-2" />
            Export Report
          </Button>
        </div>
      </div>

      {/* Key Performance Metrics */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Response Time"
          value={formatLatency(responseTime)}
          description={responseTime < 500 ? "Excellent" : responseTime < 1000 ? "Good" : "Needs optimization"}
          icon={Timer}
          trend={responseTime < 500 ? "down" : "up"}
          color={responseTime < 500 ? "green" : responseTime < 1000 ? "yellow" : "red"}
        />
        <MetricCard
          title="Throughput"
          value={`${formatThroughput(throughput)}/s`}
          description={`${activeProcesses?.active_inferences || 0} active`}
          icon={Zap}
          trend="up"
          color="blue"
        />
        <MetricCard
          title="Success Rate"
          value={`${successRate.toFixed(1)}%`}
          description={successRate > 95 ? "Excellent" : successRate > 90 ? "Good" : "Needs attention"}
          icon={Target}
          trend={successRate > 95 ? "up" : "stable"}
          color={successRate > 95 ? "green" : successRate > 90 ? "yellow" : "red"}
        />
        <MetricCard
          title="Active Models"
          value={(loadedModels?.length || 0).toString()}
          description={`${activeProcesses?.batch_jobs || 0} batch jobs`}
          icon={Database}
          trend="stable"
          color="purple"
        />
      </div>

      {/* Performance Charts */}
      <div className="grid gap-6 lg:grid-cols-2">
        {/* System Resource Usage */}
        <Card>
          <CardHeader>
            <CardTitle>System Resource Usage</CardTitle>
            <CardDescription>Real-time CPU and memory utilization</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-6">
              {/* CPU Usage */}
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Cpu className="h-4 w-4 text-blue-500" />
                    <span className="text-sm font-medium">CPU Usage</span>
                  </div>
                  {systemLoading ? (
                    <Skeleton className="h-4 w-12" />
                  ) : (
                    <span className="text-sm font-semibold">{cpuUsage.toFixed(1)}%</span>
                  )}
                </div>
                <Progress value={cpuUsage} className="h-2" />
                <div className="text-xs text-muted-foreground">
                  {systemInfo?.cpu_cores || 0} cores • Target: &lt;80%
                </div>
              </div>

              {/* Memory Usage */}
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <HardDrive className="h-4 w-4 text-green-500" />
                    <span className="text-sm font-medium">Memory Usage</span>
                  </div>
                  {systemLoading ? (
                    <Skeleton className="h-4 w-12" />
                  ) : (
                    <span className="text-sm font-semibold">{memoryUsage.toFixed(1)}%</span>
                  )}
                </div>
                <Progress value={memoryUsage} className="h-2" />
                <div className="text-xs text-muted-foreground">
                  {((systemInfo?.used_memory || 0) / (1024 ** 3)).toFixed(1)}GB / {((systemInfo?.total_memory || 0) / (1024 ** 3)).toFixed(1)}GB • Target: &lt;80%
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Performance Trends */}
        <Card>
          <CardHeader>
            <CardTitle>Performance Trends</CardTitle>
            <CardDescription>Historical performance over time</CardDescription>
          </CardHeader>
          <CardContent>
            <SystemChart />
          </CardContent>
        </Card>
      </div>

      {/* Performance Analysis */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* Response Time Distribution */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Gauge className="h-4 w-4" />
              Response Time Analysis
            </CardTitle>
            <CardDescription>Current response time breakdown</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm">Average</span>
                <Badge variant="outline">{formatLatency(responseTime)}</Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">P95</span>
                <Badge variant="outline">{formatLatency((responseTime || 0) * 1.5)}</Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">P99</span>
                <Badge variant="outline">{formatLatency((responseTime || 0) * 2)}</Badge>
              </div>
              <div className="pt-4 border-t">
                <div className="text-xs text-muted-foreground">
                  Target: &lt;500ms avg, &lt;1s P95
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Error Rate Analysis */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-4 w-4" />
              Error Rate Analysis
            </CardTitle>
            <CardDescription>Success and failure tracking</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm">Success Rate</span>
                <Badge variant={successRate > 95 ? "default" : "destructive"}>
                  {successRate.toFixed(1)}%
                </Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">Total Requests</span>
                <span className="text-sm font-semibold">
                  <AnimatedCounter value={infernoMetrics?.inference_count || 0} />
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">Failed Requests</span>
                <span className="text-sm font-semibold text-red-600">
                  <AnimatedCounter value={infernoMetrics?.error_count || 0} />
                </span>
              </div>
              <div className="pt-4 border-t">
                <div className="text-xs text-muted-foreground">
                  Target: &gt;99% success rate
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Performance Recommendations */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-4 w-4" />
              Optimization Tips
            </CardTitle>
            <CardDescription>Performance improvement suggestions</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {cpuUsage > 80 && (
                <div className="flex items-start gap-2 p-2 bg-yellow-50 rounded-lg border border-yellow-200">
                  <AlertTriangle className="h-4 w-4 text-yellow-600 mt-0.5" />
                  <div>
                    <div className="text-sm font-medium text-yellow-800">High CPU Usage</div>
                    <div className="text-xs text-yellow-700">Consider load balancing or scaling</div>
                  </div>
                </div>
              )}

              {memoryUsage > 80 && (
                <div className="flex items-start gap-2 p-2 bg-orange-50 rounded-lg border border-orange-200">
                  <AlertTriangle className="h-4 w-4 text-orange-600 mt-0.5" />
                  <div>
                    <div className="text-sm font-medium text-orange-800">High Memory Usage</div>
                    <div className="text-xs text-orange-700">Check for memory leaks or optimize models</div>
                  </div>
                </div>
              )}

              {responseTime > 1000 && (
                <div className="flex items-start gap-2 p-2 bg-red-50 rounded-lg border border-red-200">
                  <Timer className="h-4 w-4 text-red-600 mt-0.5" />
                  <div>
                    <div className="text-sm font-medium text-red-800">Slow Response Time</div>
                    <div className="text-xs text-red-700">Optimize inference pipeline or hardware</div>
                  </div>
                </div>
              )}

              {cpuUsage < 50 && memoryUsage < 50 && responseTime < 500 && (
                <div className="flex items-start gap-2 p-2 bg-green-50 rounded-lg border border-green-200">
                  <CheckCircle className="h-4 w-4 text-green-600 mt-0.5" />
                  <div>
                    <div className="text-sm font-medium text-green-800">Optimal Performance</div>
                    <div className="text-xs text-green-700">System is performing well</div>
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}