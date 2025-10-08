'use client';

import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useLoadedModels } from '@/hooks/use-tauri-api';
import { toast } from 'react-hot-toast';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Loader2, Send } from 'lucide-react';

interface QuickInferenceModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function QuickInferenceModal({ open, onOpenChange }: QuickInferenceModalProps) {
  const { data: loadedModels, isLoading: modelsLoading } = useLoadedModels();
  const [selectedBackend, setSelectedBackend] = useState<string>('');
  const [prompt, setPrompt] = useState('');
  const [isInferring, setIsInferring] = useState(false);
  const [response, setResponse] = useState('');
  const [streamingResponse, setStreamingResponse] = useState('');

  const handleInference = async () => {
    if (!selectedBackend || !prompt.trim()) {
      toast.error('Please select a model and enter a prompt');
      return;
    }

    setIsInferring(true);
    setResponse('');
    setStreamingResponse('');

    try {
      // Set up event listener for streaming tokens
      const unlisten = await listen<{ inference_id: string; token: string }>('inference_token', (event) => {
        setStreamingResponse(prev => prev + event.payload.token);
      });

      // Set up completion listener
      const unlistenComplete = await listen<{ inference_id: string; response: string }>('inference_complete', (event) => {
        setResponse(event.payload.response);
        setIsInferring(false);
        toast.success('Inference completed');
        unlisten();
        unlistenComplete();
      });

      // Set up error listener
      const unlistenError = await listen<{ inference_id: string; error: string }>('inference_error', (event) => {
        toast.error(`Inference failed: ${event.payload.error}`);
        setIsInferring(false);
        unlisten();
        unlistenComplete();
        unlistenError();
      });

      // Start inference
      await invoke('infer_stream', {
        backendId: selectedBackend,
        prompt,
        params: {
          temperature: 0.7,
          max_tokens: 512,
          stream: true,
        }
      });

    } catch (error) {
      console.error('Inference error:', error);
      toast.error('Failed to start inference');
      setIsInferring(false);
    }
  };

  const handleClose = () => {
    if (!isInferring) {
      onOpenChange(false);
      setPrompt('');
      setResponse('');
      setStreamingResponse('');
      setSelectedBackend('');
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Quick Inference</DialogTitle>
          <DialogDescription>
            Run a quick inference test with your loaded model
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Model Selection */}
          <div className="space-y-2">
            <Label htmlFor="model">Model</Label>
            <Select
              value={selectedBackend}
              onValueChange={setSelectedBackend}
              disabled={isInferring || modelsLoading}
            >
              <SelectTrigger id="model">
                <SelectValue placeholder={modelsLoading ? 'Loading models...' : 'Select a model'} />
              </SelectTrigger>
              <SelectContent>
                {loadedModels && loadedModels.length > 0 ? (
                  loadedModels.map((model) => (
                    <SelectItem key={model} value={model}>
                      {model}
                    </SelectItem>
                  ))
                ) : (
                  <SelectItem value="none" disabled>
                    No models loaded
                  </SelectItem>
                )}
              </SelectContent>
            </Select>
          </div>

          {/* Prompt Input */}
          <div className="space-y-2">
            <Label htmlFor="prompt">Prompt</Label>
            <Textarea
              id="prompt"
              placeholder="Enter your prompt here..."
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              disabled={isInferring}
              rows={4}
              className="resize-none"
            />
          </div>

          {/* Response Output */}
          {(streamingResponse || response) && (
            <div className="space-y-2">
              <Label>Response</Label>
              <div className="rounded-md border bg-muted p-4 min-h-[150px] max-h-[300px] overflow-y-auto">
                <pre className="whitespace-pre-wrap text-sm">
                  {isInferring ? streamingResponse : response}
                  {isInferring && <span className="animate-pulse">▊</span>}
                </pre>
              </div>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={handleClose}
            disabled={isInferring}
          >
            Close
          </Button>
          <Button
            onClick={handleInference}
            disabled={isInferring || !selectedBackend || !prompt.trim()}
          >
            {isInferring ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                Inferring...
              </>
            ) : (
              <>
                <Send className="h-4 w-4 mr-2" />
                Run Inference
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
