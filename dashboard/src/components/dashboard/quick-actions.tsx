'use client';

import { Button } from '@/components/ui/button';
import {
  Play,
  Upload,
  BarChart3,
  Settings,
  Download,
  Refresh,
  Brain,
  Clock,
} from 'lucide-react';

export function QuickActions() {
  const actions = [
    {
      title: 'Run Inference',
      description: 'Test model with custom input',
      icon: Play,
      color: 'bg-green-500 hover:bg-green-600',
    },
    {
      title: 'Upload Model',
      description: 'Add new AI model',
      icon: Upload,
      color: 'bg-blue-500 hover:bg-blue-600',
    },
    {
      title: 'View Metrics',
      description: 'System performance',
      icon: BarChart3,
      color: 'bg-purple-500 hover:bg-purple-600',
    },
    {
      title: 'Batch Job',
      description: 'Schedule bulk processing',
      icon: Clock,
      color: 'bg-orange-500 hover:bg-orange-600',
    },
    {
      title: 'Model Hub',
      description: 'Browse available models',
      icon: Brain,
      color: 'bg-pink-500 hover:bg-pink-600',
    },
    {
      title: 'Export Data',
      description: 'Download reports',
      icon: Download,
      color: 'bg-gray-500 hover:bg-gray-600',
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
        <Button variant="outline" className="w-full">
          <Refresh className="h-4 w-4 mr-2" />
          Refresh All Data
        </Button>
      </div>
    </div>
  );
}