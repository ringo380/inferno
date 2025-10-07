'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { MetricCard } from './metric-card';
import { RecentActivity } from './recent-activity';
import { SystemChart } from './system-chart';
import { ModelStatus } from './model-status';
import { QuickActions } from './quick-actions';
import { useDashboardData, useSystemInfo, useMetrics, useInfernoMetrics, useActiveProcesses } from '@/hooks/use-tauri-api';
import { MetricsToggle, useMetricsScope } from '@/components/ui/metrics-toggle';
import { Skeleton } from '@/components/ui/skeleton';
import { AnimatedCounter } from '@/components/ui/animated-counter';
import { ErrorBoundary } from '@/components/ui/error-boundary';
import { useRouter } from 'next/navigation';
// import { PageTransition, StaggerContainer, div } from '@/components/ui/page-transition';
import type { ReactNode } from 'react';
import {
  Brain,
  Cpu,
  HardDrive,
  Activity,
  Clock,
  Users,
  Zap,
  AlertTriangle,
  TrendingUp,
  Play,
} from 'lucide-react';

function formatMemorySize(bytes: number | undefined): string {
  const gb = (bytes || 0) / (1024 * 1024 * 1024);
  return `${gb.toFixed(1)} GB`;
}

function formatCpuUsage(usage: number | undefined): string {
  return `${(usage || 0).toFixed(1)}%`;
}

export function DashboardOverviewTauri() {
  const { systemInfo, metrics, isLoading, error } = useDashboardData();
  const { metricsScope, setMetricsScope, isInfernoMode } = useMetricsScope();
  const { data: infernoMetrics, isLoading: infernoLoading } = useInfernoMetrics();
  const { data: activeProcesses } = useActiveProcesses();
  const router = useRouter();

  // Use appropriate metrics based on toggle
  const currentMetrics = isInfernoMode ? infernoMetrics : metrics?.data;
  const isMetricsLoading = isInfernoMode ? infernoLoading : metrics?.isLoading;
  const streamingSessions = activeProcesses?.streaming_sessions ?? 0;
  const metricCards: ReactNode[] = [];

  if (!isLoading && !isMetricsLoading) {
    metricCards.push(
      <MetricCard
        key="metric-active-models"
        title={isInfernoMode ? 'Active Models' : 'Models Loaded'}
        value={
          <AnimatedCounter
            value={isInfernoMode ? (currentMetrics?.active_models || 0) : (currentMetrics?.models_loaded || 0)}
          />
        }
        description={isInfernoMode ? 'Inferno active models' : 'Currently loaded models'}
        icon={Brain}
        trend="stable"
        color="blue"
      />
    );

    metricCards.push(
      <MetricCard
        key="metric-total-inferences"
        title="Total Inferences"
        value={<AnimatedCounter value={currentMetrics?.inference_count || 0} />}
        description={`${currentMetrics?.success_count || 0} successful`}
        icon={TrendingUp}
        trend="up"
        color="green"
      />
    );

    metricCards.push(
      <MetricCard
        key="metric-latency"
        title="Avg Response Time"
        value={<AnimatedCounter value={Math.round(currentMetrics?.average_latency || 0)} suffix="ms" />}
        description="Average inference latency"
        icon={Zap}
        trend="stable"
        color="yellow"
      />
    );

    metricCards.push(
      <MetricCard
        key="metric-cpu"
        title={isInfernoMode ? 'Inferno CPU' : 'System CPU'}
        value={
          <AnimatedCounter
            value={isInfernoMode ? (currentMetrics?.cpu_usage || 0) : (systemInfo?.data?.cpu_usage || 0)}
            suffix="%"
          />
        }
        description={isInfernoMode ? 'Inferno process usage' : (systemInfo?.data?.platform || 'System usage')}
        icon={Cpu}
        trend="stable"
        color="blue"
      />
    );

    if (isInfernoMode && typeof infernoMetrics?.gpu_usage === 'number') {
      const gpuValue = Number.isFinite(infernoMetrics.gpu_usage)
        ? `${infernoMetrics.gpu_usage.toFixed(1)}%`
        : '0%';

      metricCards.push(
        <MetricCard
          key="metric-gpu"
          title="GPU Utilization"
          value={gpuValue}
          description="Current GPU load"
          icon={Activity}
          trend="stable"
          color="purple"
        />
      );
    }

    if (isInfernoMode) {
      metricCards.push(
        <MetricCard
          key="metric-streaming"
          title="Streaming Sessions"
          value={<AnimatedCounter value={streamingSessions} />}
          description="Active streaming outputs"
          icon={Play}
          trend={streamingSessions > 0 ? 'up' : 'stable'}
          color="green"
        />
      );
    }
  }

  if (error) {
    return (
      <div className="space-y-6">
        <Card className="border-destructive">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertTriangle className="h-5 w-5" />
              Error Loading Dashboard
            </CardTitle>
            <CardDescription>
              Failed to connect to Inferno backend. Please ensure the application is running.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              {error instanceof Error ? error.message : 'Unknown error occurred'}
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Dashboard Overview</h1>
          <p className="text-muted-foreground">
            Welcome to your Inferno AI/ML platform command center
          </p>
        </div>
        <div className="flex items-center space-x-4">
          <MetricsToggle
            value={metricsScope}
            onChange={setMetricsScope}
            className="hidden sm:flex"
          />
          <Badge variant={systemInfo?.data ? 'success' : 'secondary'} className="flex items-center gap-1">
            <Activity className="h-3 w-3" />
            {systemInfo?.data ? 'All Systems Operational' : 'Connecting...'}
          </Badge>
          <Button onClick={() => router.push('/inference')}>
            <Play className="h-4 w-4 mr-2" />
            Run Inference
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4 xl:grid-cols-5">
        {isLoading || isMetricsLoading
          ? Array.from({ length: 4 }).map((_, i) => (
              <Card key={`metric-skeleton-${i}`}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <Skeleton className="h-4 w-24" />
                  <Skeleton className="h-4 w-4" />
                </CardHeader>
                <CardContent>
                  <Skeleton className="h-8 w-16 mb-2" />
                  <Skeleton className="h-3 w-20" />
                </CardContent>
              </Card>
            ))
          : metricCards}
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>{isInfernoMode ? 'Inferno Performance' : 'System Performance'}</CardTitle>
            <CardDescription>
              {isInfernoMode
                ? 'Real-time Inferno-specific metrics and inference performance'
                : 'Real-time system metrics and inference performance'}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ErrorBoundary>
              <SystemChart />
            </ErrorBoundary>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
            <CardDescription>Common tasks and operations</CardDescription>
          </CardHeader>
          <CardContent>
            <ErrorBoundary>
              <QuickActions />
            </ErrorBoundary>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Model Status</CardTitle>
            <CardDescription>Current status of your AI models</CardDescription>
          </CardHeader>
          <CardContent>
            <ErrorBoundary>
              <ModelStatus />
            </ErrorBoundary>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
            <CardDescription>Latest system events and operations</CardDescription>
          </CardHeader>
          <CardContent>
            <ErrorBoundary>
              <RecentActivity />
            </ErrorBoundary>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
            <Cpu className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <Skeleton className="h-8 w-16" />
            ) : (
              <>
                <div className="text-2xl font-bold">
                  {systemInfo?.data ? formatCpuUsage(systemInfo.data.cpu_usage) : '0%'}
                </div>
                <div className="w-full bg-secondary rounded-full h-2 mt-2">
                  <div
                    className="bg-blue-500 h-2 rounded-full"
                    style={{ width: `${systemInfo?.data?.cpu_usage || 0}%` }}
                  ></div>
                </div>
                <p className="text-xs text-muted-foreground mt-2">
                  {systemInfo?.data ? `${systemInfo.data.cpu_cores} cores • ${systemInfo.data.arch}` : 'Loading...'}
                </p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Memory Usage</CardTitle>
            <HardDrive className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <Skeleton className="h-8 w-16" />
            ) : (
              <>
                <div className="text-2xl font-bold">
                  {systemInfo?.data ? formatMemorySize(systemInfo.data.used_memory) : '0 GB'}
                </div>
                <div className="w-full bg-secondary rounded-full h-2 mt-2">
                  <div
                    className="bg-green-500 h-2 rounded-full"
                    style={{
                      width: `${systemInfo?.data ? (systemInfo.data.used_memory / systemInfo.data.total_memory) * 100 : 0}%`
                    }}
                  ></div>
                </div>
                <p className="text-xs text-muted-foreground mt-2">
                  {systemInfo?.data
                    ? `${Math.round((systemInfo.data.used_memory / systemInfo.data.total_memory) * 100)}% of ${formatMemorySize(systemInfo.data.total_memory)}`
                    : 'Loading...'}
                </p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Error Rate</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <Skeleton className="h-8 w-16" />
            ) : (
              <>
                <div className="text-2xl font-bold">{metrics?.data?.error_count || 0}</div>
                <div className="flex items-center space-x-2 mt-2">
                  <Badge variant="success" className="text-xs">
                    {metrics?.data?.success_count || 0} Success
                  </Badge>
                  {(metrics?.data?.error_count || 0) > 0 && (
                    <Badge variant="destructive" className="text-xs">
                      {metrics?.data?.error_count} Errors
                    </Badge>
                  )}
                </div>
                <p className="text-xs text-muted-foreground mt-2">
                  Total inferences: {metrics?.data?.inference_count || 0}
                </p>
              </>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
