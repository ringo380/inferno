'use client';

import { MainLayout } from '@/components/layout/main-layout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { useBatchJobs, useSystemInfo } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Database,
  ArrowRight,
  Play,
  Pause,
  Square,
  RefreshCw,
  Plus,
  Settings,
  Download,
  Upload,
  Filter,
  MoreHorizontal,
  CheckCircle,
  AlertTriangle,
  Clock,
  FileText,
  Workflow,
  Zap,
} from 'lucide-react';
import { useState } from 'react';

// Mock pipeline data
const mockPipelines = [
  {
    id: 'pipeline_1',
    name: 'Data Preprocessing Pipeline',
    description: 'Clean and prepare training data for model fine-tuning',
    status: 'running',
    progress: 67,
    stages: ['Extract', 'Transform', 'Validate', 'Load'],
    current_stage: 2,
    created_at: '2024-01-20T10:00:00Z',
    updated_at: '2024-01-20T14:22:00Z',
    runtime: '4h 22m',
    throughput: '1.2K records/sec',
    processed_items: 45000,
    total_items: 67000,
    error_count: 23
  },
  {
    id: 'pipeline_2',
    name: 'Model Evaluation Pipeline',
    description: 'Automated model quality assessment and benchmarking',
    status: 'completed',
    progress: 100,
    stages: ['Load Model', 'Run Tests', 'Generate Report', 'Archive'],
    current_stage: 3,
    created_at: '2024-01-20T08:00:00Z',
    updated_at: '2024-01-20T12:15:00Z',
    runtime: '4h 15m',
    throughput: '850 tests/min',
    processed_items: 12000,
    total_items: 12000,
    error_count: 0
  },
  {
    id: 'pipeline_3',
    name: 'Data Export Pipeline',
    description: 'Export processed results to external systems',
    status: 'failed',
    progress: 45,
    stages: ['Prepare', 'Export', 'Verify', 'Notify'],
    current_stage: 1,
    created_at: '2024-01-20T13:00:00Z',
    updated_at: '2024-01-20T14:30:00Z',
    runtime: '1h 30m',
    throughput: '0 files/sec',
    processed_items: 3200,
    total_items: 7100,
    error_count: 15
  }
];

function getStatusColor(status: string): string {
  switch (status) {
    case 'running':
      return 'text-blue-600 bg-blue-50 border-blue-200';
    case 'completed':
      return 'text-green-600 bg-green-50 border-green-200';
    case 'failed':
      return 'text-red-600 bg-red-50 border-red-200';
    case 'paused':
      return 'text-yellow-600 bg-yellow-50 border-yellow-200';
    default:
      return 'text-gray-600 bg-gray-50 border-gray-200';
  }
}

function getStatusIcon(status: string) {
  switch (status) {
    case 'running':
      return Play;
    case 'completed':
      return CheckCircle;
    case 'failed':
      return AlertTriangle;
    case 'paused':
      return Pause;
    default:
      return Clock;
  }
}

function formatDuration(duration: string): string {
  return duration;
}

export default function PipelinePage() {
  const [selectedPipeline, setSelectedPipeline] = useState<string | null>(null);
  const { data: batchJobs, isLoading: jobsLoading } = useBatchJobs();
  const { data: systemInfo, isLoading: systemLoading } = useSystemInfo();

  const runningPipelines = mockPipelines.filter(p => p.status === 'running').length;
  const totalThroughput = mockPipelines
    .filter(p => p.status === 'running')
    .reduce((sum, p) => sum + parseFloat(p.throughput.replace(/[^\d.]/g, '')), 0);

  const handleStartPipeline = (pipelineId: string) => {
    console.log('Starting pipeline:', pipelineId);
  };

  const handlePausePipeline = (pipelineId: string) => {
    console.log('Pausing pipeline:', pipelineId);
  };

  const handleStopPipeline = (pipelineId: string) => {
    if (confirm('Are you sure you want to stop this pipeline?')) {
      console.log('Stopping pipeline:', pipelineId);
    }
  };

  return (
    <MainLayout>
      <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Data Pipeline</h1>
          <p className="text-muted-foreground">
            Manage data processing workflows and ETL operations
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant="outline" className="flex items-center gap-1">
            <Workflow className="h-3 w-3" />
            {runningPipelines} Running
          </Badge>
          <Button>
            <Plus className="h-4 w-4 mr-2" />
            New Pipeline
          </Button>
        </div>
      </div>

      {/* Pipeline Overview */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Pipelines</CardTitle>
            <Workflow className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{runningPipelines}</div>
            <p className="text-xs text-muted-foreground">
              {mockPipelines.length} total pipelines
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Throughput</CardTitle>
            <Zap className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{totalThroughput.toFixed(1)}K</div>
            <p className="text-xs text-muted-foreground">
              records per second
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <CheckCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">95.2%</div>
            <p className="text-xs text-muted-foreground">
              last 24 hours
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Data Processed</CardTitle>
            <Database className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">2.4TB</div>
            <p className="text-xs text-muted-foreground">
              this month
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Pipeline List */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Pipeline Workflows</CardTitle>
              <CardDescription>Monitor and manage data processing pipelines</CardDescription>
            </div>
            <div className="flex items-center gap-2">
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
          <div className="space-y-4">
            {mockPipelines.map((pipeline) => {
              const StatusIcon = getStatusIcon(pipeline.status);
              const statusClass = getStatusColor(pipeline.status);

              return (
                <div key={pipeline.id} className="border rounded-lg p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <div className={`p-1 rounded border ${statusClass}`}>
                          <StatusIcon className="h-4 w-4" />
                        </div>
                        <div>
                          <h3 className="font-semibold">{pipeline.name}</h3>
                          <p className="text-sm text-muted-foreground">{pipeline.description}</p>
                        </div>
                        <Badge variant="outline" className="text-xs">
                          {pipeline.status}
                        </Badge>
                      </div>

                      {/* Pipeline Progress */}
                      <div className="mb-3">
                        <div className="flex items-center justify-between mb-1">
                          <span className="text-sm font-medium">Progress</span>
                          <span className="text-sm">{pipeline.progress}%</span>
                        </div>
                        <Progress value={pipeline.progress} className="h-2" />
                        <div className="text-xs text-muted-foreground mt-1">
                          {pipeline.processed_items.toLocaleString()} / {pipeline.total_items.toLocaleString()} items
                        </div>
                      </div>

                      {/* Pipeline Stages */}
                      <div className="mb-3">
                        <div className="flex items-center gap-2">
                          {pipeline.stages.map((stage, index) => (
                            <div key={stage} className="flex items-center">
                              <div className={`px-2 py-1 rounded text-xs ${
                                index < pipeline.current_stage
                                  ? 'bg-green-100 text-green-700'
                                  : index === pipeline.current_stage
                                    ? 'bg-blue-100 text-blue-700'
                                    : 'bg-gray-100 text-gray-500'
                              }`}>
                                {stage}
                              </div>
                              {index < pipeline.stages.length - 1 && (
                                <ArrowRight className="h-3 w-3 mx-1 text-muted-foreground" />
                              )}
                            </div>
                          ))}
                        </div>
                      </div>

                      {/* Pipeline Stats */}
                      <div className="grid grid-cols-4 gap-4 text-sm">
                        <div>
                          <span className="text-muted-foreground">Runtime:</span>
                          <div className="font-medium">{formatDuration(pipeline.runtime)}</div>
                        </div>
                        <div>
                          <span className="text-muted-foreground">Throughput:</span>
                          <div className="font-medium">{pipeline.throughput}</div>
                        </div>
                        <div>
                          <span className="text-muted-foreground">Errors:</span>
                          <div className={`font-medium ${pipeline.error_count > 0 ? 'text-red-600' : 'text-green-600'}`}>
                            {pipeline.error_count}
                          </div>
                        </div>
                        <div>
                          <span className="text-muted-foreground">Updated:</span>
                          <div className="font-medium">
                            {new Date(pipeline.updated_at).toLocaleTimeString()}
                          </div>
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-2 ml-4">
                      {pipeline.status === 'running' ? (
                        <>
                          <Button
                            variant="outline"
                            size="icon"
                            onClick={() => handlePausePipeline(pipeline.id)}
                          >
                            <Pause className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="outline"
                            size="icon"
                            onClick={() => handleStopPipeline(pipeline.id)}
                          >
                            <Square className="h-4 w-4" />
                          </Button>
                        </>
                      ) : pipeline.status === 'paused' || pipeline.status === 'failed' ? (
                        <Button
                          variant="outline"
                          size="icon"
                          onClick={() => handleStartPipeline(pipeline.id)}
                        >
                          <Play className="h-4 w-4" />
                        </Button>
                      ) : null}

                      <Button variant="outline" size="icon">
                        <Settings className="h-4 w-4" />
                      </Button>
                      <Button variant="outline" size="icon">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Pipeline Templates</CardTitle>
            <CardDescription>Pre-configured workflows for common tasks</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {[
                { name: 'Data Ingestion', description: 'Import data from external sources', icon: Upload },
                { name: 'Data Transformation', description: 'Clean and transform raw data', icon: RefreshCw },
                { name: 'Model Training', description: 'Train ML models on processed data', icon: Zap },
                { name: 'Result Export', description: 'Export results to target systems', icon: Download }
              ].map((template) => {
                const Icon = template.icon;
                return (
                  <div key={template.name} className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center gap-3">
                      <Icon className="h-5 w-5 text-primary" />
                      <div>
                        <div className="font-medium">{template.name}</div>
                        <div className="text-sm text-muted-foreground">{template.description}</div>
                      </div>
                    </div>
                    <Button variant="outline" size="sm">
                      Use Template
                    </Button>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Resource Usage</CardTitle>
            <CardDescription>Current pipeline resource consumption</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">CPU Usage</span>
                  <span className="text-sm">{systemLoading ? '...' : `${(systemInfo?.cpu_usage || 0).toFixed(1)}%`}</span>
                </div>
                <Progress value={systemInfo?.cpu_usage || 0} className="h-2" />
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Memory Usage</span>
                  <span className="text-sm">
                    {systemLoading ? '...' : `${((systemInfo?.used_memory || 0) / (1024 ** 3)).toFixed(1)}GB`}
                  </span>
                </div>
                <Progress
                  value={systemInfo ? (systemInfo.used_memory / systemInfo.total_memory) * 100 : 0}
                  className="h-2"
                />
              </div>

              <div className="flex items-center justify-between text-sm">
                <span>Active Workers</span>
                <span className="font-semibold">{runningPipelines * 2}</span>
              </div>

              <div className="flex items-center justify-between text-sm">
                <span>Queue Length</span>
                <span className="font-semibold">3</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
      </div>
    </MainLayout>
  );
}