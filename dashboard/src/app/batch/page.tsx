'use client';

import { useState } from 'react';
import { MainLayout } from '@/components/layout/main-layout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Textarea } from '@/components/ui/textarea';
import {
  Plus,
  Play,
  Pause,
  Square,
  Trash2,
  Upload,
  Download,
  Clock,
  CheckCircle,
  AlertTriangle,
  FileText,
  Calendar,
  BarChart3,
  Settings,
  RefreshCw,
} from 'lucide-react';
import { toast } from 'react-hot-toast';
import {
  useBatchJobs,
  useCreateBatchJob,
  useStartBatchJob,
  usePauseBatchJob,
  useCancelBatchJob,
  useDeleteBatchJob,
  useModels
} from '@/hooks/use-tauri-api';
import { BatchJob } from '@/types/inferno';
import { Skeleton } from '@/components/ui/skeleton';


export default function BatchPage() {
  // Fetch real data
  const { data: jobs = [], isLoading: jobsLoading, error: jobsError, refetch: refetchJobs } = useBatchJobs();
  const { data: availableModels = [] } = useModels();

  // Mutations
  const createJobMutation = useCreateBatchJob();
  const startJobMutation = useStartBatchJob();
  const pauseJobMutation = usePauseBatchJob();
  const cancelJobMutation = useCancelBatchJob();
  const deleteJobMutation = useDeleteBatchJob();

  // Local state
  const [selectedJob, setSelectedJob] = useState<BatchJob | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [newJobName, setNewJobName] = useState('');
  const [newJobModel, setNewJobModel] = useState('');
  const [newJobPrompts, setNewJobPrompts] = useState('');

  const getStatusColor = (status: BatchJob['status']) => {
    switch (status) {
      case 'completed':
        return 'text-green-600 bg-green-100 dark:text-green-400 dark:bg-green-900/20';
      case 'running':
        return 'text-blue-600 bg-blue-100 dark:text-blue-400 dark:bg-blue-900/20';
      case 'failed':
        return 'text-red-600 bg-red-100 dark:text-red-400 dark:bg-red-900/20';
      case 'cancelled':
        return 'text-orange-600 bg-orange-100 dark:text-orange-400 dark:bg-orange-900/20';
      default: // pending
        return 'text-gray-600 bg-gray-100 dark:text-gray-400 dark:bg-gray-900/20';
    }
  };

  const handleCreateJob = () => {
    if (!newJobName.trim() || !newJobPrompts.trim()) {
      toast.error('Please fill in all required fields');
      return;
    }

    if (!newJobModel) {
      toast.error('Please select a model');
      return;
    }

    const prompts = newJobPrompts.split('\n').filter(p => p.trim());
    if (prompts.length === 0) {
      toast.error('Please add at least one prompt');
      return;
    }

    const jobData = {
      name: newJobName,
      model_id: newJobModel,
      inputs: prompts,
      output_format: 'text',
      batch_size: 5,
      parallel_workers: 2,
    };

    createJobMutation.mutate(jobData, {
      onSuccess: () => {
        setNewJobName('');
        setNewJobModel('');
        setNewJobPrompts('');
        setIsCreating(false);
      }
    });
  };

  const handleStartJob = (jobId: string) => {
    startJobMutation.mutate(jobId);
  };

  const handlePauseJob = (jobId: string) => {
    pauseJobMutation.mutate(jobId);
  };

  const handleStopJob = (jobId: string) => {
    cancelJobMutation.mutate(jobId);
  };

  const handleDeleteJob = (jobId: string) => {
    deleteJobMutation.mutate(jobId, {
      onSuccess: () => {
        if (selectedJob?.id === jobId) {
          setSelectedJob(null);
        }
      }
    });
  };

  const handleExportResults = (job: BatchJob) => {
    if (!job.results || !job.results.outputs || job.results.outputs.length === 0) {
      toast.error('No results to export');
      return;
    }

    const data = {
      jobName: job.name,
      modelId: job.model_id,
      createdAt: job.created_at,
      completedAt: job.completed_at,
      inputs: job.config.inputs,
      outputs: job.results.outputs,
      errors: job.results.errors || [],
      metrics: job.results.metrics,
      config: job.config,
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `batch-results-${job.name.replace(/\s+/g, '-')}-${Date.now()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    toast.success('Results exported successfully');
  };

  const handleImportPrompts = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.txt,.csv,.json';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = (e) => {
        const content = e.target?.result as string;
        if (file.name.endsWith('.json')) {
          try {
            const data = JSON.parse(content);
            if (Array.isArray(data)) {
              setNewJobPrompts(data.join('\n'));
            } else if (data.prompts && Array.isArray(data.prompts)) {
              setNewJobPrompts(data.prompts.join('\n'));
            }
          } catch (error) {
            toast.error('Invalid JSON format');
          }
        } else {
          setNewJobPrompts(content);
        }
        toast.success('Prompts imported successfully');
      };
      reader.readAsText(file);
    };
    input.click();
  };

  return (
    <MainLayout>
      <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Batch Processing</h1>
          <p className="text-muted-foreground">
            Manage and execute batch inference jobs
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button variant="outline" onClick={() => refetchJobs()}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
          <Button onClick={() => setIsCreating(true)}>
            <Plus className="h-4 w-4 mr-2" />
            New Job
          </Button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Jobs</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{jobs.length}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Running</CardTitle>
            <Play className="h-4 w-4 text-blue-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-blue-600">
              {jobs.filter(j => j.status === 'running').length}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Completed</CardTitle>
            <CheckCircle className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">
              {jobs.filter(j => j.status === 'completed').length}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Pending</CardTitle>
            <Clock className="h-4 w-4 text-gray-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-gray-600">
              {jobs.filter(j => j.status === 'pending').length}
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Job List */}
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle>Batch Jobs</CardTitle>
              <CardDescription>
                Manage your batch processing jobs
              </CardDescription>
            </CardHeader>
            <CardContent>
              {jobsLoading ? (
                <div className="space-y-4">
                  {[1, 2, 3].map((i) => (
                    <div key={i} className="p-4 border rounded-lg">
                      <div className="flex items-center justify-between mb-2">
                        <Skeleton className="h-5 w-32" />
                        <Skeleton className="h-5 w-16" />
                      </div>
                      <Skeleton className="h-4 w-48 mb-2" />
                      <Skeleton className="h-3 w-40" />
                    </div>
                  ))}
                </div>
              ) : jobs.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  <FileText className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p className="text-sm">No batch jobs yet</p>
                  <p className="text-xs">Create your first batch job to get started</p>
                </div>
              ) : (
                <div className="space-y-4">
                  {jobs.map((job) => (
                    <div
                      key={job.id}
                      className={`p-4 border rounded-lg cursor-pointer transition-colors ${
                        selectedJob?.id === job.id ? 'border-primary bg-primary/5' : 'hover:bg-accent'
                      }`}
                      onClick={() => setSelectedJob(job)}
                    >
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center space-x-2">
                          <h4 className="font-medium">{job.name}</h4>
                          <Badge className={getStatusColor(job.status)}>
                            {job.status}
                          </Badge>
                          {job.schedule && (
                            <Badge variant="outline" className="text-xs">
                              <Calendar className="h-3 w-3 mr-1" />
                              Scheduled
                            </Badge>
                          )}
                        </div>
                        <div className="flex items-center space-x-1">
                          {job.status === 'pending' && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={(e) => {
                                e.stopPropagation();
                                handleStartJob(job.id);
                              }}
                            >
                              <Play className="h-4 w-4" />
                            </Button>
                          )}
                          {job.status === 'running' && (
                            <>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handlePauseJob(job.id);
                                }}
                              >
                                <Pause className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleStopJob(job.id);
                                }}
                              >
                                <Square className="h-4 w-4" />
                              </Button>
                            </>
                          )}
                          {job.status === 'completed' && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={(e) => {
                                e.stopPropagation();
                                handleExportResults(job);
                              }}
                            >
                              <Download className="h-4 w-4" />
                            </Button>
                          )}
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleDeleteJob(job.id);
                            }}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </div>

                      <div className="text-sm text-muted-foreground mb-2">
                        Model: {job.model_id} • {job.config.inputs.length} prompts
                      </div>

                      {job.status === 'running' && (
                        <div className="space-y-1">
                          <div className="flex justify-between text-xs">
                            <span>Progress</span>
                            <span>{job.progress}%</span>
                          </div>
                          <Progress value={job.progress} className="h-2" />
                        </div>
                      )}

                      <div className="text-xs text-muted-foreground mt-2">
                        Created: {new Date(job.created_at).toLocaleString()}
                        {job.completed_at && (
                          <> • Completed: {new Date(job.completed_at).toLocaleString()}</>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Job Details / Create Form */}
        <div>
          {isCreating ? (
            <Card>
              <CardHeader>
                <CardTitle>Create New Job</CardTitle>
                <CardDescription>
                  Set up a new batch processing job
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Job Name</label>
                  <input
                    type="text"
                    value={newJobName}
                    onChange={(e) => setNewJobName(e.target.value)}
                    className="w-full px-3 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500"
                    placeholder="Enter job name"
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">Model</label>
                  <select
                    value={newJobModel}
                    onChange={(e) => setNewJobModel(e.target.value)}
                    className="w-full px-3 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500"
                  >
                    <option value="">Select a model</option>
                    {availableModels.map((model) => (
                      <option key={model.name} value={model.name}>
                        {model.name} ({model.backend_type})
                      </option>
                    ))}
                  </select>
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <label className="text-sm font-medium">Prompts</label>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleImportPrompts}
                    >
                      <Upload className="h-4 w-4 mr-2" />
                      Import
                    </Button>
                  </div>
                  <Textarea
                    value={newJobPrompts}
                    onChange={(e) => setNewJobPrompts(e.target.value)}
                    placeholder="Enter prompts (one per line)"
                    className="min-h-[120px]"
                  />
                  <p className="text-xs text-muted-foreground">
                    Enter one prompt per line. You can also import from a file.
                  </p>
                </div>

                <div className="flex space-x-2">
                  <Button onClick={handleCreateJob} className="flex-1">
                    Create Job
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => setIsCreating(false)}
                  >
                    Cancel
                  </Button>
                </div>
              </CardContent>
            </Card>
          ) : selectedJob ? (
            <Card>
              <CardHeader>
                <CardTitle>Job Details</CardTitle>
                <CardDescription>
                  {selectedJob.name}
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Status:</span>
                    <Badge className={getStatusColor(selectedJob.status)}>
                      {selectedJob.status}
                    </Badge>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Model:</span>
                    <span className="text-sm font-medium">{selectedJob.model_id}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Prompts:</span>
                    <span className="text-sm font-medium">{selectedJob.config.inputs.length}</span>
                  </div>
                  {selectedJob.progress > 0 && (
                    <div className="space-y-1">
                      <div className="flex justify-between text-sm">
                        <span className="text-muted-foreground">Progress:</span>
                        <span className="font-medium">{selectedJob.progress}%</span>
                      </div>
                      <Progress value={selectedJob.progress} className="h-2" />
                    </div>
                  )}
                </div>

                <div className="space-y-2">
                  <h4 className="text-sm font-medium">Prompts</h4>
                  <div className="space-y-1 max-h-40 overflow-y-auto">
                    {selectedJob.config.inputs.map((prompt, index) => (
                      <div key={index} className="p-2 text-xs bg-accent rounded">
                        {prompt}
                      </div>
                    ))}
                  </div>
                </div>

                {selectedJob.results && selectedJob.results.outputs && (
                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <h4 className="text-sm font-medium">Results</h4>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleExportResults(selectedJob)}
                      >
                        <Download className="h-4 w-4 mr-2" />
                        Export
                      </Button>
                    </div>
                    <div className="space-y-1 max-h-40 overflow-y-auto">
                      {selectedJob.results.outputs.map((result, index) => (
                        <div key={index} className="p-2 text-xs bg-accent rounded">
                          <div className="font-medium mb-1">Result {index + 1}:</div>
                          {result}
                        </div>
                      ))}
                    </div>
                    {selectedJob.results.errors && selectedJob.results.errors.length > 0 && (
                      <div className="space-y-1">
                        <h5 className="text-xs font-semibold text-destructive">Errors</h5>
                        <div className="space-y-1 max-h-32 overflow-y-auto">
                          {selectedJob.results.errors.map((msg, index) => (
                            <div key={`error-${index}`} className="p-2 text-xs bg-destructive/10 text-destructive rounded">
                              {msg}
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                    {selectedJob.results.metrics && (
                      <div className="grid grid-cols-3 gap-2 text-xs text-muted-foreground">
                        <div className="p-2 bg-accent/40 rounded">
                          <div className="font-medium text-foreground">Total Time</div>
                          <div>{selectedJob.results.metrics.total_time.toFixed(2)}s</div>
                        </div>
                        <div className="p-2 bg-accent/40 rounded">
                          <div className="font-medium text-foreground">Avg / Task</div>
                          <div>{selectedJob.results.metrics.avg_time_per_task.toFixed(2)}s</div>
                        </div>
                        <div className="p-2 bg-accent/40 rounded">
                          <div className="font-medium text-foreground">Throughput</div>
                          <div>{selectedJob.results.metrics.throughput.toFixed(2)} req/s</div>
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </CardContent>
            </Card>
          ) : (
            <Card>
              <CardContent className="pt-6">
                <div className="text-center py-8 text-muted-foreground">
                  <BarChart3 className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p className="text-sm">Select a job to view details</p>
                  <p className="text-xs">Or create a new batch job to get started</p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
      </div>
    </MainLayout>
  );
}
