'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { MetricCard } from './metric-card';
import { RecentActivity } from './recent-activity';
import { SystemChart } from './system-chart';
import { ModelStatus } from './model-status';
import { QuickActions } from './quick-actions';
import { Skeleton } from '@/components/ui/skeleton';
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
import {
  useSystemInfo,
  useInfernoMetrics,
  useLoadedModels,
  useActiveProcesses,
  useBatchJobCount,
  useActiveBatchJobCount
} from '@/hooks/use-tauri-api';
import {
  useRealTimeSystemMetrics,
  useRealTimeEvents,
  useRealTimeModelEvents,
  useRealTimeInferenceEvents
} from '@/hooks/use-real-time-events';
import { ConnectionStatus, EventStreamDebugger } from '@/components/ui/connection-status';

export function DashboardOverview() {
  // Original polling-based hooks for fallback
  const { data: systemInfo, isLoading: systemLoading } = useSystemInfo();
  const { data: infernoMetrics, isLoading: metricsLoading } = useInfernoMetrics();
  const { data: loadedModels, isLoading: modelsLoading } = useLoadedModels();
  const { data: activeProcesses, isLoading: processesLoading } = useActiveProcesses();
  const { data: batchJobCount, isLoading: batchCountLoading } = useBatchJobCount();
  const { data: activeBatchJobCount, isLoading: activeBatchCountLoading } = useActiveBatchJobCount();

  // Debug logging
  console.log('📊 Dashboard Data:', {
    systemInfo,
    infernoMetrics,
    loadedModels,
    activeProcesses,
    isLoading: systemLoading || metricsLoading
  });

  // Real-time event hooks
  const { metrics: realTimeSystemMetrics } = useRealTimeSystemMetrics();
  const { connectionStatus } = useRealTimeEvents();
  const { modelEvents } = useRealTimeModelEvents();
  const { activeInferenceCount } = useRealTimeInferenceEvents();

  // Use real-time metrics if available, otherwise fall back to polling
  const currentSystemMetrics = realTimeSystemMetrics || {
    cpu_usage: systemInfo?.cpu_usage || 0,
    memory_usage: systemInfo?.used_memory || 0,
  };

  const streamingSessions = infernoMetrics?.active_streaming_sessions ?? activeProcesses?.streaming_sessions ?? 0;
  const gpuUsage =
    typeof infernoMetrics?.gpu_usage === 'number' && Number.isFinite(infernoMetrics.gpu_usage)
      ? infernoMetrics.gpu_usage
      : undefined;

  const isLoading =
    systemLoading ||
    metricsLoading ||
    modelsLoading ||
    processesLoading ||
    batchCountLoading ||
    activeBatchCountLoading;

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Dashboard Overview</h1>
          <p className="text-muted-foreground">
            Welcome to your Inferno AI/ML platform command center
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <ConnectionStatus compact />
          <Badge
            variant={connectionStatus === 'Connected' ? 'success' : 'destructive'}
            className="flex items-center gap-1.5"
          >
            {connectionStatus === 'Connected' ? (
              <>
                <span className="relative flex h-2 w-2">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
                </span>
                Real-time Active
              </>
            ) : (
              <>
                <Activity className="h-3 w-3" />
                Polling Mode
              </>
            )}
          </Badge>
          <Button>
            <Play className="h-4 w-4 mr-2" />
            Run Inference
          </Button>
        </div>
      </div>

      {/* Key Metrics Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Active Models"
          value={modelsLoading ? "..." : (loadedModels?.length || 0).toString()}
          description={modelsLoading ? "Loading..." : `${loadedModels?.length || 0} models loaded`}
          icon={Brain}
          trend="stable"
          color="blue"
        />
        <MetricCard
          title="Total Inferences"
          value={metricsLoading ? "..." : (infernoMetrics?.inference_count?.toLocaleString() || "0")}
          description={
            metricsLoading
              ? "Loading..."
              : `${activeInferenceCount} active (real-time) • ${streamingSessions} streaming`
          }
          icon={TrendingUp}
          trend="up"
          color="green"
        />
        <MetricCard
          title="Avg Response Time"
          value={metricsLoading ? "..." : `${infernoMetrics?.avg_response_time_ms || 0}ms`}
          description={metricsLoading ? "Loading..." : "Real-time average"}
          icon={Zap}
          trend="stable"
          color="yellow"
        />
        <MetricCard
          title="System Load"
          value={systemLoading ? "..." : `${currentSystemMetrics.cpu_usage?.toFixed(1) || 0}%`}
          description={
            realTimeSystemMetrics
              ? `Real-time CPU${gpuUsage !== undefined ? ` • GPU ${gpuUsage.toFixed(1)}%` : ''}`
              : `CPU utilization${gpuUsage !== undefined ? ` • GPU ${gpuUsage.toFixed(1)}%` : ''}`
          }
          icon={Cpu}
          trend="stable"
          color="blue"
        />
      </div>

      {/* Main Content Grid */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* System Performance Chart */}
        <Card className="lg:col-span-2">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>System Performance</CardTitle>
                <CardDescription>
                  Real-time system metrics over the last 24 hours
                </CardDescription>
              </div>
              {realTimeSystemMetrics && (
                <Badge variant="outline" className="flex items-center gap-1 bg-green-50 dark:bg-green-950 border-green-200 dark:border-green-800 text-green-700 dark:text-green-400">
                  <span className="relative flex h-2 w-2">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                    <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
                  </span>
                  Updating
                </Badge>
              )}
            </div>
          </CardHeader>
          <CardContent>
            <SystemChart />
          </CardContent>
        </Card>

        {/* Quick Actions */}
        <Card>
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
            <CardDescription>Common tasks and operations</CardDescription>
          </CardHeader>
          <CardContent>
            <QuickActions />
          </CardContent>
        </Card>
      </div>

      {/* Secondary Content Grid */}
      <div className="grid gap-6 lg:grid-cols-2">
        {/* Model Status */}
        <Card>
          <CardHeader>
            <CardTitle>Model Status</CardTitle>
            <CardDescription>Current status of your AI models</CardDescription>
          </CardHeader>
          <CardContent>
            <ModelStatus />
          </CardContent>
        </Card>

        {/* Recent Activity */}
        <Card>
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
            <CardDescription>Latest system events and operations</CardDescription>
          </CardHeader>
          <CardContent>
            <RecentActivity />
          </CardContent>
        </Card>
      </div>

      {/* System Resource Overview */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
            <Cpu className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {systemLoading ? (
              <Skeleton className="h-8 w-16 mb-2" />
            ) : (
              <div className="text-2xl font-bold flex items-center gap-2 transition-all duration-300">
                {currentSystemMetrics.cpu_usage?.toFixed(1) || 0}%
                {realTimeSystemMetrics && (
                  <span className="text-xs bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-400 px-1.5 py-0.5 rounded flex items-center gap-1 animate-pulse">
                    <span className="inline-block w-1.5 h-1.5 bg-green-600 dark:bg-green-400 rounded-full animate-ping"></span>
                    LIVE
                  </span>
                )}
              </div>
            )}
            <div className="w-full bg-secondary rounded-full h-2 mt-2 overflow-hidden">
              <div
                className="bg-blue-500 h-2 rounded-full transition-all duration-500 ease-out"
                style={{ width: `${Math.min(currentSystemMetrics.cpu_usage || 0, 100)}%` }}
              ></div>
            </div>
            <div className="text-xs text-muted-foreground mt-2">
              {systemLoading ? (
                <Skeleton className="h-3 w-32" />
              ) : (
                `${systemInfo?.cpu_cores || 0} cores • ${systemInfo?.cpu_frequency || '0'}GHz`
              )}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Memory Usage</CardTitle>
            <HardDrive className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {systemLoading ? (
              <Skeleton className="h-8 w-20 mb-2" />
            ) : (
              <div className="text-2xl font-bold flex items-center gap-2 transition-all duration-300">
                {((currentSystemMetrics.memory_usage || 0) / (1024 ** 3)).toFixed(1)} GB
                {realTimeSystemMetrics && (
                  <span className="text-xs bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-400 px-1.5 py-0.5 rounded flex items-center gap-1 animate-pulse">
                    <span className="inline-block w-1.5 h-1.5 bg-green-600 dark:bg-green-400 rounded-full animate-ping"></span>
                    LIVE
                  </span>
                )}
              </div>
            )}
            <div className="w-full bg-secondary rounded-full h-2 mt-2 overflow-hidden">
              <div
                className="bg-green-500 h-2 rounded-full transition-all duration-500 ease-out"
                style={{
                  width: `${Math.min(
                    ((currentSystemMetrics.memory_usage || 0) / (systemInfo?.total_memory || 1)) * 100,
                    100
                  )}%`,
                }}
              ></div>
            </div>
            <div className="text-xs text-muted-foreground mt-2">
              {systemLoading ? (
                <Skeleton className="h-3 w-40" />
              ) : (
                `${(((currentSystemMetrics.memory_usage || 0) / (systemInfo?.total_memory || 1)) * 100).toFixed(0)}% of ${((systemInfo?.total_memory || 0) / (1024 ** 3)).toFixed(0)}GB`
              )}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Jobs</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {batchCountLoading || activeBatchCountLoading ? (
              <Skeleton className="h-8 w-8 mb-2" />
            ) : (
              <div className="text-2xl font-bold">{batchJobCount || 0}</div>
            )}
            <div className="flex items-center space-x-2 mt-2">
              {batchCountLoading || activeBatchCountLoading ? (
                <>
                  <Skeleton className="h-5 w-16" />
                  <Skeleton className="h-5 w-16" />
                </>
              ) : (
                <>
                  <Badge variant="success" className="text-xs">
                    {activeBatchJobCount || 0} Running
                  </Badge>
                  <Badge variant="secondary" className="text-xs">
                    {(batchJobCount || 0) - (activeBatchJobCount || 0)} Queued
                  </Badge>
                </>
              )}
            </div>
            <div className="text-xs text-muted-foreground mt-2">
              {batchCountLoading ? (
                <Skeleton className="h-3 w-32" />
              ) : (
                `${batchJobCount || 0} total jobs • ${infernoMetrics?.active_inferences || 0} inferences • ${streamingSessions} streaming`
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Event Stream Debugger (development only) */}
      <EventStreamDebugger />
    </div>
  );
}
