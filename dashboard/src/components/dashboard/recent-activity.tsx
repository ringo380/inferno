'use client';

import { Badge } from '@/components/ui/badge';
import {
  Brain,
  Play,
  Upload,
  AlertTriangle,
  CheckCircle,
  Clock,
  User,
  Settings,
  RefreshCw,
} from 'lucide-react';
import { formatRelativeTime } from '@/lib/utils';
import { useRecentActivities } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';

// Helper function to get icon based on activity type
const getActivityIcon = (activityType: string) => {
  switch (activityType.toLowerCase()) {
    case 'inference':
      return Play;
    case 'modelload':
    case 'modelupload':
      return Upload;
    case 'modelunload':
      return Brain;
    case 'error':
      return AlertTriangle;
    case 'system':
      return Settings;
    default:
      return CheckCircle;
  }
};

// Helper function to get status color
const getStatusColor = (status: string) => {
  switch (status.toLowerCase()) {
    case 'success':
      return 'text-green-600 bg-green-100 dark:text-green-400 dark:bg-green-900/20';
    case 'warning':
      return 'text-yellow-600 bg-yellow-100 dark:text-yellow-400 dark:bg-yellow-900/20';
    case 'error':
      return 'text-red-600 bg-red-100 dark:text-red-400 dark:bg-red-900/20';
    case 'inprogress':
      return 'text-blue-600 bg-blue-100 dark:text-blue-400 dark:bg-blue-900/20';
    default:
      return 'text-blue-600 bg-blue-100 dark:text-blue-400 dark:bg-blue-900/20';
  }
};

export function RecentActivity() {
  const { data: activities, isLoading, error } = useRecentActivities(10);

  if (isLoading) {
    return (
      <div className="space-y-4">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="flex items-start space-x-3">
            <Skeleton className="h-7 w-7 rounded-full" />
            <div className="flex-1 space-y-2">
              <Skeleton className="h-4 w-48" />
              <Skeleton className="h-3 w-32" />
              <Skeleton className="h-3 w-20" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center p-8 border rounded-lg border-destructive/20">
        <div className="text-center space-y-2">
          <AlertTriangle className="h-8 w-8 text-destructive mx-auto" />
          <p className="text-sm text-muted-foreground">
            Failed to load activities: {error.message}
          </p>
        </div>
      </div>
    );
  }

  if (!activities || activities.length === 0) {
    return (
      <div className="flex items-center justify-center p-8 border rounded-lg border-dashed">
        <div className="text-center space-y-2">
          <Clock className="h-8 w-8 text-muted-foreground mx-auto" />
          <p className="text-sm text-muted-foreground">
            No recent activities
          </p>
          <p className="text-xs text-muted-foreground">
            Activities will appear here as you use the system
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {activities.map((activity) => {
        const Icon = getActivityIcon(activity.activity_type);
        return (
          <div key={activity.id} className="flex items-start space-x-3">
            <div className={`p-2 rounded-full ${getStatusColor(activity.status)}`}>
              <Icon className="h-3 w-3" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between">
                <p className="text-sm font-medium text-foreground">
                  {activity.title}
                </p>
                <p className="text-xs text-muted-foreground">
                  {formatRelativeTime(activity.timestamp)}
                </p>
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                {activity.description}
              </p>
              <div className="flex items-center space-x-2 mt-1">
                <User className="h-3 w-3 text-muted-foreground" />
                <span className="text-xs text-muted-foreground">
                  {activity.user}
                </span>
              </div>
            </div>
          </div>
        );
      })}

      <div className="pt-2 border-t text-center">
        <button className="text-sm text-primary hover:underline">
          View All Activity
        </button>
      </div>
    </div>
  );
}