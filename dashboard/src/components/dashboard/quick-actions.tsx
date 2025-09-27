'use client';

import { Button } from '@/components/ui/button';
import {
  Play,
  Upload,
  BarChart3,
  Settings,
  Download,
  RefreshCw,
  Brain,
  Clock,
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useDashboardData, useActiveProcesses, useInfernoMetrics, useRecentActivities } from '@/hooks/use-tauri-api';
import { useQueryClient } from '@tanstack/react-query';
import { toast } from 'react-hot-toast';

export function QuickActions() {
  const router = useRouter();
  const queryClient = useQueryClient();

  // Fetch data for export functionality
  const dashboardData = useDashboardData();
  const { data: activeProcesses } = useActiveProcesses();
  const { data: infernoMetrics } = useInfernoMetrics();
  const { data: recentActivities } = useRecentActivities(100);

  const handleRefreshAll = async () => {
    try {
      await queryClient.invalidateQueries();
      toast.success('All data refreshed successfully');
    } catch (error) {
      console.error('Refresh failed:', error);
      toast.error('Failed to refresh data');
    }
  };

  const handleExportData = async () => {
    try {
      const exportData = {
        timestamp: new Date().toISOString(),
        system_info: dashboardData.systemInfo.data,
        metrics: {
          dashboard: dashboardData.metrics.data,
          inferno: infernoMetrics,
        },
        models: {
          available: dashboardData.models.data,
          loaded: dashboardData.loadedModels.data,
        },
        active_processes: activeProcesses,
        recent_activities: recentActivities,
        metadata: {
          export_version: '1.0',
          platform: navigator.platform,
          user_agent: navigator.userAgent,
        },
      };

      const dataStr = JSON.stringify(exportData, null, 2);
      const dataBlob = new Blob([dataStr], { type: 'application/json' });

      const url = URL.createObjectURL(dataBlob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `inferno-dashboard-export-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      toast.success('Dashboard data exported successfully');
    } catch (error) {
      console.error('Export failed:', error);
      toast.error('Failed to export data');
    }
  };

  const actions = [
    {
      title: 'Run Inference',
      description: 'Test model with custom input',
      icon: Play,
      color: 'bg-green-500 hover:bg-green-600',
      onClick: () => router.push('/inference'),
    },
    {
      title: 'Upload Model',
      description: 'Add new AI model',
      icon: Upload,
      color: 'bg-blue-500 hover:bg-blue-600',
      onClick: () => router.push('/models'),
    },
    {
      title: 'View Metrics',
      description: 'System performance',
      icon: BarChart3,
      color: 'bg-purple-500 hover:bg-purple-600',
      onClick: () => router.push('/monitoring'),
    },
    {
      title: 'Batch Job',
      description: 'Schedule bulk processing',
      icon: Clock,
      color: 'bg-orange-500 hover:bg-orange-600',
      onClick: () => router.push('/batch-jobs'),
    },
    {
      title: 'Model Hub',
      description: 'Browse available models',
      icon: Brain,
      color: 'bg-pink-500 hover:bg-pink-600',
      onClick: () => router.push('/models'),
    },
    {
      title: 'Export Data',
      description: 'Download reports',
      icon: Download,
      color: 'bg-gray-500 hover:bg-gray-600',
      onClick: handleExportData,
    },
  ];

  return (
    <div className="space-y-3">
      {actions.map((action, index) => {
        const Icon = action.icon;
        return (
          <Button
            key={index}
            variant="ghost"
            className="w-full justify-start h-auto p-3 hover:bg-accent"
            onClick={action.onClick}
          >
            <div className={`p-2 rounded-md ${action.color} mr-3`}>
              <Icon className="h-4 w-4 text-white" />
            </div>
            <div className="text-left">
              <div className="font-medium text-sm">{action.title}</div>
              <div className="text-xs text-muted-foreground">
                {action.description}
              </div>
            </div>
          </Button>
        );
      })}

      <div className="pt-3 border-t">
        <Button variant="outline" className="w-full" onClick={handleRefreshAll}>
          <RefreshCw className="h-4 w-4 mr-2" />
          Refresh All Data
        </Button>
      </div>
    </div>
  );
}