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
} from 'lucide-react';
import { formatRelativeTime } from '@/lib/utils';

const mockActivities = [
  {
    id: '1',
    type: 'inference',
    title: 'Inference completed',
    description: 'Llama 2 7B processed 150 tokens in 2.3s',
    timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
    status: 'success',
    icon: Play,
    user: 'john.doe',
  },
  {
    id: '2',
    type: 'model',
    title: 'Model uploaded',
    description: 'New model "GPT-4 Turbo" added to library',
    timestamp: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
    status: 'success',
    icon: Upload,
    user: 'sarah.wilson',
  },
  {
    id: '3',
    type: 'warning',
    title: 'High memory usage',
    description: 'System memory usage reached 85%',
    timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
    status: 'warning',
    icon: AlertTriangle,
    user: 'system',
  },
  {
    id: '4',
    type: 'job',
    title: 'Batch job completed',
    description: 'Processed 1,000 text samples successfully',
    timestamp: new Date(Date.now() - 45 * 60 * 1000).toISOString(),
    status: 'success',
    icon: CheckCircle,
    user: 'batch-processor',
  },
  {
    id: '5',
    type: 'model',
    title: 'Model quantization',
    description: 'Llama 2 13B quantized to Q4_0 format',
    timestamp: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
    status: 'success',
    icon: Brain,
    user: 'admin',
  },
  {
    id: '6',
    type: 'config',
    title: 'Settings updated',
    description: 'Cache settings modified by administrator',
    timestamp: new Date(Date.now() - 90 * 60 * 1000).toISOString(),
    status: 'info',
    icon: Settings,
    user: 'admin',
  },
];

const getStatusColor = (status: string) => {
  switch (status) {
    case 'success':
      return 'text-green-600 bg-green-100 dark:text-green-400 dark:bg-green-900/20';
    case 'warning':
      return 'text-yellow-600 bg-yellow-100 dark:text-yellow-400 dark:bg-yellow-900/20';
    case 'error':
      return 'text-red-600 bg-red-100 dark:text-red-400 dark:bg-red-900/20';
    default:
      return 'text-blue-600 bg-blue-100 dark:text-blue-400 dark:bg-blue-900/20';
  }
};

export function RecentActivity() {
  return (
    <div className="space-y-4">
      {mockActivities.map((activity) => {
        const Icon = activity.icon;
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