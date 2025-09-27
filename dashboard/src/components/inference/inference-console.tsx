'use client';

import { useState, useRef, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ModelSelector } from './model-selector';
import { InferenceParameters } from './inference-parameters';
import { InferenceHistory } from './inference-history';
import { StreamingOutput } from './streaming-output';
import {
  Play,
  Square,
  Trash2,
  Save,
  Share,
  Settings,
  Clock,
  Zap,
  RotateCcw,
} from 'lucide-react';
import { InferenceParams, InferenceResponse } from '@/types/inferno';
import { useInference, useLoadedModels } from '@/hooks/use-tauri-api';
import { toast } from 'react-hot-toast';

const examplePrompts = [
  "Write a short story about AI in the future",
  "Explain quantum computing in simple terms",
  "Create a Python function to sort a list",
  "What are the benefits of renewable energy?",
  "Write a haiku about machine learning",
];

export function InferenceConsole() {
  const { data: loadedModels, isLoading: modelsLoading } = useLoadedModels();
  const inferMutation = useInference();

  const [selectedModel, setSelectedModel] = useState('');
  const [prompt, setPrompt] = useState('');
  const [response, setResponse] = useState('');
  const [currentParams, setCurrentParams] = useState<InferenceParams>({
    temperature: 0.7,
    top_k: 40,
    top_p: 0.9,
    max_tokens: 512,
    stream: true,
  });
  const [history, setHistory] = useState<InferenceResponse[]>([]);
  const [metrics, setMetrics] = useState({
    tokensGenerated: 0,
    inferenceTime: 0,
    tokensPerSecond: 0,
  });

  // Auto-select first loaded model when models are available
  useEffect(() => {
    if (loadedModels && loadedModels.length > 0 && !selectedModel) {
      setSelectedModel(loadedModels[0]);
    }
  }, [loadedModels, selectedModel]);

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!prompt.trim() || inferMutation.isPending || !selectedModel) return;

    if (!selectedModel) {
      toast.error('Please select a model first');
      return;
    }

    setResponse('');
    setMetrics({ tokensGenerated: 0, inferenceTime: 0, tokensPerSecond: 0 });

    const startTime = Date.now();

    try {
      const result = await inferMutation.mutateAsync({
        backendId: selectedModel,
        prompt: prompt.trim(),
        params: currentParams,
      });

      const endTime = Date.now();
      const inferenceTime = endTime - startTime;
      const tokenCount = result.length; // Rough estimate
      const tokensPerSecond = tokenCount / (inferenceTime / 1000);

      setResponse(result);
      setMetrics({
        tokensGenerated: tokenCount,
        inferenceTime,
        tokensPerSecond,
      });

      // Add to history
      const newResponse: InferenceResponse = {
        id: Math.random().toString(36).substr(2, 9),
        model_id: selectedModel,
        prompt,
        response: result,
        tokens_generated: tokenCount,
        inference_time_ms: inferenceTime,
        tokens_per_second: tokensPerSecond,
        timestamp: new Date().toISOString(),
        status: 'success',
      };

      setHistory(prev => [newResponse, ...prev]);
      toast.success('Inference completed successfully');

    } catch (error) {
      console.error('Inference error:', error);
      toast.error('Inference failed: ' + (error instanceof Error ? error.message : 'Unknown error'));
      setResponse('Error: Failed to generate response');
    }
  };

  const stopInference = () => {
    // Note: Currently Tauri doesn't support canceling in-progress inference
    // This would need to be implemented in the backend
    toast('Inference cancellation not yet supported');
  };

  const clearConsole = () => {
    setPrompt('');
    setResponse('');
    setMetrics({ tokensGenerated: 0, inferenceTime: 0, tokensPerSecond: 0 });
  };

  const loadExample = (example: string) => {
    setPrompt(example);
    textareaRef.current?.focus();
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Inference Console</h1>
          <p className="text-muted-foreground">
            Test and interact with your AI models in real-time
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant={inferMutation.isPending ? 'default' : 'secondary'} className="flex items-center gap-1">
            {inferMutation.isPending ? (
              <>
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                Running
              </>
            ) : (
              <>
                <div className="w-2 h-2 bg-gray-500 rounded-full" />
                Ready
              </>
            )}
          </Badge>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Main Console */}
        <div className="lg:col-span-2 space-y-6">
          {/* Model Selection */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Model Configuration</CardTitle>
              <CardDescription>
                Select your model and configure inference parameters
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <ModelSelector
                selectedModel={selectedModel}
                onModelChange={setSelectedModel}
              />
              <InferenceParameters
                params={currentParams}
                onChange={setCurrentParams}
              />
            </CardContent>
          </Card>

          {/* Input Area */}
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-lg">Prompt Input</CardTitle>
                  <CardDescription>
                    Enter your prompt or select from examples
                  </CardDescription>
                </div>
                <div className="flex items-center space-x-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={clearConsole}
                    disabled={inferMutation.isPending}
                  >
                    <RotateCcw className="h-4 w-4 mr-2" />
                    Clear
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Example Prompts */}
              <div className="space-y-2">
                <label className="text-sm font-medium text-muted-foreground">
                  Example Prompts
                </label>
                <div className="flex flex-wrap gap-2">
                  {examplePrompts.map((example, index) => (
                    <Button
                      key={index}
                      variant="outline"
                      size="sm"
                      onClick={() => loadExample(example)}
                      disabled={inferMutation.isPending}
                      className="text-xs"
                    >
                      {example.slice(0, 30)}...
                    </Button>
                  ))}
                </div>
              </div>

              {/* Prompt Textarea */}
              <form onSubmit={handleSubmit} className="space-y-4">
                <textarea
                  ref={textareaRef}
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  placeholder="Enter your prompt here..."
                  className="w-full h-32 p-3 border rounded-lg bg-background resize-none focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                  disabled={inferMutation.isPending}
                />
                <div className="flex items-center justify-between">
                  <div className="text-sm text-muted-foreground">
                    {prompt.length} characters
                  </div>
                  <div className="flex items-center space-x-2">
                    {inferMutation.isPending && (
                      <Button
                        type="button"
                        variant="outline"
                        onClick={stopInference}
                      >
                        <Square className="h-4 w-4 mr-2" />
                        Stop
                      </Button>
                    )}
                    <Button
                      type="submit"
                      disabled={!prompt.trim() || inferMutation.isPending}
                    >
                      <Play className="h-4 w-4 mr-2" />
                      {inferMutation.isPending ? 'Generating...' : 'Run Inference'}
                    </Button>
                  </div>
                </div>
              </form>
            </CardContent>
          </Card>

          {/* Output Area */}
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-lg">Output</CardTitle>
                  <CardDescription>
                    Model response and generation metrics
                  </CardDescription>
                </div>
                <div className="flex items-center space-x-2">
                  <Button variant="outline" size="sm" disabled={!response}>
                    <Save className="h-4 w-4 mr-2" />
                    Save
                  </Button>
                  <Button variant="outline" size="sm" disabled={!response}>
                    <Share className="h-4 w-4 mr-2" />
                    Share
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <StreamingOutput
                response={response}
                isStreaming={inferMutation.isPending}
                metrics={metrics}
              />
            </CardContent>
          </Card>
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          {/* Real-time Metrics */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Performance Metrics</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 gap-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Clock className="h-4 w-4 text-muted-foreground" />
                    <span className="text-sm">Time</span>
                  </div>
                  <span className="font-medium">
                    {(metrics.inferenceTime / 1000).toFixed(1)}s
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Zap className="h-4 w-4 text-muted-foreground" />
                    <span className="text-sm">Tokens/sec</span>
                  </div>
                  <span className="font-medium">
                    {metrics.tokensPerSecond.toFixed(1)}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Settings className="h-4 w-4 text-muted-foreground" />
                    <span className="text-sm">Tokens</span>
                  </div>
                  <span className="font-medium">
                    {metrics.tokensGenerated}
                  </span>
                </div>
              </div>

              {inferMutation.isPending && (
                <div className="pt-3 border-t">
                  <div className="flex items-center space-x-2 text-sm text-muted-foreground">
                    <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                    <span>Generating response...</span>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Model Info */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Model Information</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div>
                <div className="text-sm text-muted-foreground">Model</div>
                <div className="font-medium">Llama 2 7B Q4_0</div>
              </div>
              <div>
                <div className="text-sm text-muted-foreground">Parameters</div>
                <div className="font-medium">7.0B</div>
              </div>
              <div>
                <div className="text-sm text-muted-foreground">Context Length</div>
                <div className="font-medium">4,096 tokens</div>
              </div>
              <div>
                <div className="text-sm text-muted-foreground">Status</div>
                <Badge variant="success" className="text-xs">Loaded</Badge>
              </div>
            </CardContent>
          </Card>

          {/* Inference History */}
          <InferenceHistory history={history} onLoadPrompt={setPrompt} />
        </div>
      </div>
    </div>
  );
}