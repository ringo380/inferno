'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Brain, MoreHorizontal } from 'lucide-react';
import { getStatusColor, formatBytes } from '@/lib/utils';

const mockModels = [
  {
    id: 'llama-7b',
    name: 'Llama 2 7B',
    status: 'loaded',
    size: 13631488000,
    format: 'gguf',
    usage: 85,
    lastUsed: '2 minutes ago',
  },
  {
    id: 'gpt-3.5-turbo',
    name: 'GPT-3.5 Turbo',
    status: 'available',
    size: 6442450944,
    format: 'onnx',
    usage: 0,
    lastUsed: '1 hour ago',
  },
  {
    id: 'claude-instant',
    name: 'Claude Instant',
    status: 'loading',
    size: 8589934592,
    format: 'gguf',
    usage: 45,
    lastUsed: 'Loading...',
  },
  {
    id: 'mistral-7b',
    name: 'Mistral 7B',
    status: 'error',
    size: 12884901888,
    format: 'gguf',
    usage: 0,
    lastUsed: '30 minutes ago',
  },
];

export function ModelStatus() {
  return (
    <div className="space-y-4">
      {mockModels.map((model) => (
        <div
          key={model.id}
          className="flex items-center justify-between p-4 border rounded-lg hover:bg-accent/50 transition-colors"
        >
          <div className="flex items-center space-x-3">
            <div className="p-2 rounded-md bg-primary/10">
              <Brain className="h-4 w-4 text-primary" />
            </div>
            <div>
              <div className="font-medium text-sm">{model.name}</div>
              <div className="flex items-center space-x-2 text-xs text-muted-foreground">
                <span>{formatBytes(model.size)}</span>
                <span>•</span>
                <span className="uppercase">{model.format}</span>
                <span>•</span>
                <span>{model.lastUsed}</span>
              </div>
            </div>
          </div>

          <div className="flex items-center space-x-3">
            {/* Status Badge */}
            <Badge className={getStatusColor(model.status)}>
              {model.status}
            </Badge>

            {/* Usage Progress (for loaded models) */}
            {model.status === 'loaded' && (
              <div className="flex items-center space-x-2">
                <span className="text-xs text-muted-foreground w-8">
                  {model.usage}%
                </span>
                <Progress value={model.usage} className="w-16 h-2" />
              </div>
            )}

            {/* Loading Progress */}
            {model.status === 'loading' && (
              <div className="flex items-center space-x-2">
                <span className="text-xs text-muted-foreground w-8">
                  {model.usage}%
                </span>
                <Progress value={model.usage} className="w-16 h-2" />
              </div>
            )}

            {/* Actions */}
            <Button variant="ghost" size="icon" className="h-8 w-8">
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </div>
        </div>
      ))}

      <div className="pt-2 border-t text-center">
        <Button variant="outline" size="sm">
          View All Models
        </Button>
      </div>
    </div>
  );
}