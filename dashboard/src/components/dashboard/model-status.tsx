'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Brain, MoreHorizontal, RefreshCw, AlertCircle } from 'lucide-react';
import { getStatusColor, formatBytes } from '@/lib/utils';
import { useModels, useLoadedModels } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';

export function ModelStatus() {
  const { data: models, isLoading: modelsLoading, error: modelsError } = useModels();
  const { data: loadedModels, isLoading: loadedLoading, error: loadedError } = useLoadedModels();

  if (modelsLoading || loadedLoading) {
    return (
      <div className="space-y-4">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="flex items-center justify-between p-4 border rounded-lg">
            <div className="flex items-center space-x-3">
              <Skeleton className="h-8 w-8 rounded-md" />
              <div className="space-y-2">
                <Skeleton className="h-4 w-32" />
                <Skeleton className="h-3 w-24" />
              </div>
            </div>
            <div className="flex items-center space-x-3">
              <Skeleton className="h-6 w-16" />
              <Skeleton className="h-8 w-8" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (modelsError || loadedError) {
    return (
      <div className="flex items-center justify-center p-8 border rounded-lg border-destructive/20">
        <div className="text-center space-y-2">
          <AlertCircle className="h-8 w-8 text-destructive mx-auto" />
          <p className="text-sm text-muted-foreground">
            Failed to load models: {(modelsError || loadedError)?.message}
          </p>
        </div>
      </div>
    );
  }

  if (!models || models.length === 0) {
    return (
      <div className="flex items-center justify-center p-8 border rounded-lg border-dashed">
        <div className="text-center space-y-2">
          <Brain className="h-8 w-8 text-muted-foreground mx-auto" />
          <p className="text-sm text-muted-foreground">
            No models found in the configured directory
          </p>
          <Button variant="outline" size="sm">
            <RefreshCw className="h-3 w-3 mr-1" />
            Refresh
          </Button>
        </div>
      </div>
    );
  }

  // Enhance models with loading status
  const enhancedModels = models.map(model => ({
    ...model,
    isLoaded: loadedModels?.includes(model.id) || false,
    status: loadedModels?.includes(model.id) ? 'loaded' : 'available',
    usage: loadedModels?.includes(model.id) ? 45 : 0, // Static value until backend provides real usage metrics
    lastUsed: loadedModels?.includes(model.id) ? 'Active' : 'Not loaded',
  }));

  return (
    <div className="space-y-4">
      {enhancedModels.map((model) => (
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