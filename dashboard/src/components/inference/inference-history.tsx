'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { InferenceResponse } from '@/types/inferno';
import { formatRelativeTime, formatDuration } from '@/lib/utils';
import { RotateCcw, Trash2, Clock, Zap } from 'lucide-react';

interface InferenceHistoryProps {
  history: InferenceResponse[];
  onLoadPrompt: (prompt: string) => void;
}

export function InferenceHistory({ history, onLoadPrompt }: InferenceHistoryProps) {
  const clearHistory = () => {
    // In a real app, this would clear the history
    console.log('Clear history');
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-lg">Inference History</CardTitle>
            <CardDescription>Recent inference runs</CardDescription>
          </div>
          {history.length > 0 && (
            <Button
              variant="outline"
              size="sm"
              onClick={clearHistory}
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Clear
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent>
        {history.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <Clock className="h-8 w-8 mx-auto mb-2 opacity-50" />
            <p className="text-sm">No inference history yet</p>
            <p className="text-xs">Run your first inference to see it here</p>
          </div>
        ) : (
          <div className="space-y-3 max-h-[400px] overflow-y-auto scrollbar-thin">
            {history.map((item) => (
              <div
                key={item.id}
                className="p-3 border rounded-lg hover:bg-accent/50 transition-colors cursor-pointer"
                onClick={() => onLoadPrompt(item.prompt)}
              >
                <div className="space-y-2">
                  {/* Header */}
                  <div className="flex items-center justify-between">
                    <Badge
                      variant={item.status === 'success' ? 'success' : 'destructive'}
                      className="text-xs"
                    >
                      {item.status}
                    </Badge>
                    <span className="text-xs text-muted-foreground">
                      {formatRelativeTime(item.timestamp)}
                    </span>
                  </div>

                  {/* Prompt Preview */}
                  <div>
                    <p className="text-sm font-medium line-clamp-2">
                      {item.prompt}
                    </p>
                  </div>

                  {/* Response Preview */}
                  <div>
                    <p className="text-xs text-muted-foreground line-clamp-2">
                      {item.response}
                    </p>
                  </div>

                  {/* Metrics */}
                  <div className="flex items-center justify-between text-xs text-muted-foreground">
                    <div className="flex items-center space-x-3">
                      <div className="flex items-center space-x-1">
                        <Clock className="h-3 w-3" />
                        <span>{formatDuration(item.inference_time_ms / 1000)}</span>
                      </div>
                      <div className="flex items-center space-x-1">
                        <Zap className="h-3 w-3" />
                        <span>{item.tokens_per_second.toFixed(1)} t/s</span>
                      </div>
                    </div>
                    <div className="text-xs">
                      {item.tokens_generated} tokens
                    </div>
                  </div>

                  {/* Action Hint */}
                  <div className="flex items-center justify-center pt-2 border-t border-muted/50">
                    <div className="flex items-center space-x-1 text-xs text-muted-foreground">
                      <RotateCcw className="h-3 w-3" />
                      <span>Click to reuse prompt</span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}