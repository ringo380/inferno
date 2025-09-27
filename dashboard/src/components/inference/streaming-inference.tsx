'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { useStreamingInference } from '@/hooks/use-streaming-inference';
import { useLoadedModels } from '@/hooks/use-tauri-api';
import { Play, Square, Loader2, AlertCircle, CheckCircle } from 'lucide-react';

export function StreamingInference() {
  const [prompt, setPrompt] = useState('Explain the concept of artificial intelligence in simple terms.');
  const [selectedBackend, setSelectedBackend] = useState<string>('');

  const { data: loadedModels, isLoading: modelsLoading } = useLoadedModels();
  const {
    isStreaming,
    currentText,
    isComplete,
    error,
    inferenceId,
    startStreaming,
    stopStreaming,
  } = useStreamingInference();

  const handleStartStreaming = async () => {
    if (!selectedBackend) return;

    try {
      await startStreaming(selectedBackend, prompt, {
        max_tokens: 100,
        temperature: 0.7,
        top_p: 0.9,
      });
    } catch (error) {
      console.error('Failed to start streaming:', error);
    }
  };

  const handleStop = () => {
    stopStreaming();
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Play className="h-5 w-5" />
            Streaming Inference
          </CardTitle>
          <CardDescription>
            Test real-time streaming inference with loaded models
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Model Selection */}
          <div className="space-y-2">
            <label className="text-sm font-medium">Select Model Backend</label>
            {modelsLoading ? (
              <div className="flex items-center gap-2">
                <Loader2 className="h-4 w-4 animate-spin" />
                <span className="text-sm text-muted-foreground">Loading models...</span>
              </div>
            ) : (
              <div className="flex flex-wrap gap-2">
                {loadedModels && loadedModels.length > 0 ? (
                  loadedModels.map((backend) => (
                    <Button
                      key={backend}
                      variant={selectedBackend === backend ? 'default' : 'outline'}
                      size="sm"
                      onClick={() => setSelectedBackend(backend)}
                    >
                      {backend}
                    </Button>
                  ))
                ) : (
                  <div className="text-sm text-muted-foreground">
                    No models loaded. Load a model first.
                  </div>
                )}
              </div>
            )}
          </div>

          <Separator />

          {/* Prompt Input */}
          <div className="space-y-2">
            <label className="text-sm font-medium">Prompt</label>
            <Textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Enter your prompt here..."
              disabled={isStreaming}
              rows={3}
            />
          </div>

          {/* Controls */}
          <div className="flex items-center gap-2">
            <Button
              onClick={handleStartStreaming}
              disabled={isStreaming || !selectedBackend || !prompt.trim()}
              className="flex items-center gap-2"
            >
              {isStreaming ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Streaming...
                </>
              ) : (
                <>
                  <Play className="h-4 w-4" />
                  Start Streaming
                </>
              )}
            </Button>

            {isStreaming && (
              <Button
                onClick={handleStop}
                variant="outline"
                className="flex items-center gap-2"
              >
                <Square className="h-4 w-4" />
                Stop
              </Button>
            )}

            {/* Status Indicators */}
            <div className="flex items-center gap-2 ml-auto">
              {isStreaming && (
                <Badge variant="secondary" className="flex items-center gap-1">
                  <Loader2 className="h-3 w-3 animate-spin" />
                  Streaming
                </Badge>
              )}
              {isComplete && (
                <Badge variant="success" className="flex items-center gap-1">
                  <CheckCircle className="h-3 w-3" />
                  Complete
                </Badge>
              )}
              {error && (
                <Badge variant="destructive" className="flex items-center gap-1">
                  <AlertCircle className="h-3 w-3" />
                  Error
                </Badge>
              )}
            </div>
          </div>

          <Separator />

          {/* Output */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium">Output</label>
              {inferenceId && (
                <Badge variant="outline" className="text-xs">
                  ID: {inferenceId.slice(0, 8)}...
                </Badge>
              )}
            </div>

            <Card className="min-h-32">
              <CardContent className="p-4">
                {error ? (
                  <div className="text-destructive text-sm">
                    <strong>Error:</strong> {error}
                  </div>
                ) : currentText ? (
                  <div className="text-sm whitespace-pre-wrap">
                    {currentText}
                    {isStreaming && (
                      <span className="animate-pulse text-primary">|</span>
                    )}
                  </div>
                ) : (
                  <div className="text-muted-foreground text-sm italic">
                    Output will appear here as it streams...
                  </div>
                )}
              </CardContent>
            </Card>
          </div>

          {/* Debug Info */}
          {process.env.NODE_ENV === 'development' && (
            <details className="text-xs">
              <summary className="cursor-pointer text-muted-foreground">Debug Info</summary>
              <pre className="mt-2 p-2 bg-muted rounded text-xs overflow-auto">
                {JSON.stringify({
                  isStreaming,
                  isComplete,
                  error,
                  inferenceId,
                  textLength: currentText.length,
                  selectedBackend,
                }, null, 2)}
              </pre>
            </details>
          )}
        </CardContent>
      </Card>
    </div>
  );
}