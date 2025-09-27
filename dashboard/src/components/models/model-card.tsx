'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import {
  Brain,
  MoreHorizontal,
  Play,
  Download,
  Trash2,
  Settings,
  Eye,
  Copy,
  Loader2,
  Square,
} from 'lucide-react';
import { ModelInfo } from '@/types/inferno';
import { formatBytes, formatTimestamp, getStatusColor } from '@/lib/utils';
import { useLoadModel, useUnloadModel, useLoadedModels } from '@/hooks/use-tauri-api';
import { toast } from 'react-hot-toast';

interface ModelCardProps {
  model: ModelInfo;
  viewMode: 'grid' | 'list';
  onSelect: (model: ModelInfo) => void;
  onQuantize: (model: ModelInfo) => void;
}

export function ModelCard({ model, viewMode, onSelect, onQuantize }: ModelCardProps) {
  const isLoading = model.status === 'loading';
  const isLoaded = model.status === 'loaded';

  const loadModelMutation = useLoadModel();
  const unloadModelMutation = useUnloadModel();
  const { data: loadedModels } = useLoadedModels();

  const handleLoadUnload = async () => {
    if (isLoaded) {
      // Find the backend ID for this model
      const backendId = loadedModels?.find(id => id.includes(model.id)) || model.id;
      try {
        await unloadModelMutation.mutateAsync(backendId);
      } catch (error) {
        console.error('Failed to unload model:', error);
      }
    } else {
      try {
        await loadModelMutation.mutateAsync({
          modelName: model.name,
          backendType: model.format === 'gguf' ? 'gguf' : 'onnx'
        });
      } catch (error) {
        console.error('Failed to load model:', error);
      }
    }
  };

  if (viewMode === 'list') {
    return (
      <Card className="card-hover">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className="p-2 rounded-md bg-primary/10">
                <Brain className="h-5 w-5 text-primary" />
              </div>
              <div>
                <h3 className="font-semibold text-sm">{model.name}</h3>
                <div className="flex items-center space-x-2 text-xs text-muted-foreground">
                  <span>{formatBytes(model.size)}</span>
                  <span>•</span>
                  <span className="uppercase">{model.format}</span>
                  {model.metadata?.quantization && (
                    <>
                      <span>•</span>
                      <span>{model.metadata.quantization}</span>
                    </>
                  )}
                  {model.metadata?.created_at && (
                    <>
                      <span>•</span>
                      <span>{formatTimestamp(model.metadata.created_at)}</span>
                    </>
                  )}
                </div>
              </div>
            </div>

            <div className="flex items-center space-x-3">
              <Badge className={getStatusColor(model.status)}>
                {model.status}
              </Badge>

              {isLoading && (
                <div className="flex items-center space-x-2">
                  <Progress value={65} className="w-16 h-2" />
                  <span className="text-xs text-muted-foreground">65%</span>
                </div>
              )}

              <div className="flex items-center space-x-1">
                <Button variant="ghost" size="sm" onClick={() => onSelect(model)}>
                  <Eye className="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={model.status === 'error' || loadModelMutation.isPending || unloadModelMutation.isPending}
                  onClick={handleLoadUnload}
                >
                  {loadModelMutation.isPending || unloadModelMutation.isPending ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : isLoaded ? (
                    <Square className="h-4 w-4" />
                  ) : (
                    <Play className="h-4 w-4" />
                  )}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onQuantize(model)}
                >
                  <Download className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="sm">
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="card-hover">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="p-2 rounded-md bg-primary/10">
            <Brain className="h-5 w-5 text-primary" />
          </div>
          <Badge className={getStatusColor(model.status)}>
            {model.status}
          </Badge>
        </div>
        <CardTitle className="text-lg">{model.name}</CardTitle>
        <CardDescription className="text-sm line-clamp-2">
          {model.metadata?.description || 'No description available'}
        </CardDescription>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Loading Progress */}
        {isLoading && (
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span>Loading...</span>
              <span>65%</span>
            </div>
            <Progress value={65} className="h-2" />
          </div>
        )}

        {/* Model Info */}
        <div className="grid grid-cols-2 gap-3 text-sm">
          <div>
            <span className="text-muted-foreground">Size:</span>
            <div className="font-medium">{formatBytes(model.size)}</div>
          </div>
          <div>
            <span className="text-muted-foreground">Format:</span>
            <div className="font-medium uppercase">{model.format}</div>
          </div>
          {model.metadata?.parameters && (
            <div>
              <span className="text-muted-foreground">Parameters:</span>
              <div className="font-medium">
                {(model.metadata.parameters / 1000000000).toFixed(1)}B
              </div>
            </div>
          )}
          {model.metadata?.quantization && (
            <div>
              <span className="text-muted-foreground">Quantization:</span>
              <div className="font-medium">{model.metadata.quantization}</div>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="flex items-center justify-between pt-2 border-t">
          <div className="flex items-center space-x-1">
            <Button variant="ghost" size="sm" onClick={() => onSelect(model)}>
              <Eye className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Copy className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Settings className="h-4 w-4" />
            </Button>
          </div>

          <div className="flex items-center space-x-1">
            <Button
              variant="outline"
              size="sm"
              disabled={model.status === 'error' || loadModelMutation.isPending || unloadModelMutation.isPending}
              onClick={handleLoadUnload}
            >
              {loadModelMutation.isPending || unloadModelMutation.isPending ? (
                <Loader2 className="h-4 w-4 mr-1 animate-spin" />
              ) : isLoaded ? (
                <Square className="h-4 w-4 mr-1" />
              ) : (
                <Play className="h-4 w-4 mr-1" />
              )}
              {loadModelMutation.isPending ? 'Loading...' :
               unloadModelMutation.isPending ? 'Unloading...' :
               isLoaded ? 'Unload' : 'Load'}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onQuantize(model)}
            >
              <Download className="h-4 w-4 mr-1" />
              Quantize
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}