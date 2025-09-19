'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Brain, ChevronDown } from 'lucide-react';
import { useState } from 'react';

interface ModelSelectorProps {
  selectedModel: string;
  onModelChange: (modelId: string) => void;
}

const availableModels = [
  {
    id: 'llama-7b-q4',
    name: 'Llama 2 7B Q4_0',
    status: 'loaded',
    parameters: '7B',
    size: '3.8GB',
    description: 'Fast and efficient for most tasks',
  },
  {
    id: 'gpt-3.5-turbo',
    name: 'GPT-3.5 Turbo',
    status: 'available',
    parameters: '175B',
    size: '6.4GB',
    description: 'Excellent for general purpose tasks',
  },
  {
    id: 'mistral-7b',
    name: 'Mistral 7B Instruct',
    status: 'error',
    parameters: '7B',
    size: '4.2GB',
    description: 'Optimized for instruction following',
  },
  {
    id: 'claude-instant',
    name: 'Claude Instant',
    status: 'loading',
    parameters: '52B',
    size: '8.5GB',
    description: 'Fast, helpful, and harmless AI assistant',
  },
];

export function ModelSelector({ selectedModel, onModelChange }: ModelSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);

  const selected = availableModels.find(m => m.id === selectedModel);
  const loadedModels = availableModels.filter(m => m.status === 'loaded');

  return (
    <div className="space-y-3">
      <label className="text-sm font-medium">Select Model</label>

      <div className="relative">
        <Button
          variant="outline"
          onClick={() => setIsOpen(!isOpen)}
          className="w-full justify-between h-auto p-3"
        >
          <div className="flex items-center space-x-3">
            <div className="p-1 rounded bg-primary/10">
              <Brain className="h-4 w-4 text-primary" />
            </div>
            <div className="text-left">
              <div className="font-medium">{selected?.name}</div>
              <div className="text-xs text-muted-foreground">
                {selected?.parameters} • {selected?.size}
              </div>
            </div>
          </div>
          <ChevronDown className="h-4 w-4" />
        </Button>

        {isOpen && (
          <div className="absolute top-full left-0 right-0 mt-1 bg-background border rounded-lg shadow-lg z-50">
            <div className="p-2 space-y-1">
              {availableModels.map((model) => (
                <button
                  key={model.id}
                  onClick={() => {
                    onModelChange(model.id);
                    setIsOpen(false);
                  }}
                  disabled={model.status === 'error'}
                  className={`w-full flex items-center justify-between p-3 rounded-md text-left hover:bg-accent disabled:opacity-50 disabled:cursor-not-allowed ${
                    selectedModel === model.id ? 'bg-accent' : ''
                  }`}
                >
                  <div className="flex items-center space-x-3">
                    <div className="p-1 rounded bg-primary/10">
                      <Brain className="h-4 w-4 text-primary" />
                    </div>
                    <div>
                      <div className="font-medium text-sm">{model.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {model.description}
                      </div>
                      <div className="text-xs text-muted-foreground mt-1">
                        {model.parameters} • {model.size}
                      </div>
                    </div>
                  </div>
                  <div className="flex flex-col items-end space-y-1">
                    <Badge
                      variant={
                        model.status === 'loaded'
                          ? 'success'
                          : model.status === 'error'
                          ? 'destructive'
                          : model.status === 'loading'
                          ? 'warning'
                          : 'secondary'
                      }
                      className="text-xs"
                    >
                      {model.status}
                    </Badge>
                  </div>
                </button>
              ))}
            </div>

            {loadedModels.length === 0 && (
              <div className="p-4 text-center text-sm text-muted-foreground border-t">
                No models are currently loaded.{' '}
                <button className="text-primary hover:underline">
                  Load a model
                </button>
              </div>
            )}
          </div>
        )}
      </div>

      {selected && selected.status !== 'loaded' && (
        <div className="text-sm text-muted-foreground">
          {selected.status === 'loading' && (
            <span>⏳ Model is loading... This may take a few minutes.</span>
          )}
          {selected.status === 'error' && (
            <span className="text-red-600">
              ❌ Model failed to load. Check logs for details.
            </span>
          )}
          {selected.status === 'available' && (
            <span>
              📥 Model is available but not loaded.{' '}
              <button className="text-primary hover:underline">
                Load now
              </button>
            </span>
          )}
        </div>
      )}
    </div>
  );
}