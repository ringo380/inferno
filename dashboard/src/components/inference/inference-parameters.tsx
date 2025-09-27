'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { InferenceParams } from '@/types/inferno';
import { Settings, RotateCcw, Zap } from 'lucide-react';
import { useState } from 'react';

interface InferenceParametersProps {
  params: InferenceParams;
  onChange: (params: InferenceParams) => void;
}

const presets = {
  creative: {
    name: 'Creative',
    description: 'High creativity, diverse outputs',
    temperature: 0.9,
    top_k: 50,
    top_p: 0.95,
    max_tokens: 512,
  },
  balanced: {
    name: 'Balanced',
    description: 'Good balance of quality and creativity',
    temperature: 0.7,
    top_k: 40,
    top_p: 0.9,
    max_tokens: 512,
  },
  precise: {
    name: 'Precise',
    description: 'Focused, deterministic outputs',
    temperature: 0.3,
    top_k: 20,
    top_p: 0.8,
    max_tokens: 512,
  },
  fast: {
    name: 'Fast',
    description: 'Quick generation, shorter responses',
    temperature: 0.5,
    top_k: 30,
    top_p: 0.85,
    max_tokens: 256,
  },
};

export function InferenceParameters({ params, onChange }: InferenceParametersProps) {
  const [showAdvanced, setShowAdvanced] = useState(false);

  const updateParam = (key: keyof InferenceParams, value: any) => {
    onChange({ ...params, [key]: value });
  };

  const applyPreset = (preset: typeof presets.balanced) => {
    onChange({
      ...params,
      temperature: preset.temperature,
      top_k: preset.top_k,
      top_p: preset.top_p,
      max_tokens: preset.max_tokens,
    });
  };

  const resetToDefaults = () => {
    onChange({
      temperature: 0.7,
      top_k: 40,
      top_p: 0.9,
      max_tokens: 512,
      stream: true,
    });
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <label className="text-sm font-medium">Inference Parameters</label>
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            onClick={resetToDefaults}
            className="text-xs"
          >
            <RotateCcw className="h-3 w-3 mr-1" />
            Reset
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowAdvanced(!showAdvanced)}
            className="text-xs"
          >
            <Settings className="h-3 w-3 mr-1" />
            {showAdvanced ? 'Simple' : 'Advanced'}
          </Button>
        </div>
      </div>

      {/* Presets */}
      <div className="space-y-2">
        <label className="text-sm text-muted-foreground">Quick Presets</label>
        <div className="grid grid-cols-2 gap-2">
          {Object.entries(presets).map(([key, preset]) => (
            <Button
              key={key}
              variant="outline"
              size="sm"
              onClick={() => applyPreset(preset)}
              className="h-auto p-2 text-left"
            >
              <div>
                <div className="font-medium text-xs">{preset.name}</div>
                <div className="text-xs text-muted-foreground">
                  {preset.description}
                </div>
              </div>
            </Button>
          ))}
        </div>
      </div>

      {/* Basic Parameters */}
      <div className="grid grid-cols-2 gap-4">
        {/* Temperature */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-sm text-muted-foreground">Temperature</label>
            <span className="text-sm font-medium">{params.temperature}</span>
          </div>
          <input
            type="range"
            min="0.1"
            max="2.0"
            step="0.1"
            value={params.temperature}
            onChange={(e) => updateParam('temperature', parseFloat(e.target.value))}
            className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>Conservative</span>
            <span>Creative</span>
          </div>
        </div>

        {/* Max Tokens */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-sm text-muted-foreground">Max Tokens</label>
            <span className="text-sm font-medium">{params.max_tokens}</span>
          </div>
          <input
            type="range"
            min="50"
            max="2048"
            step="50"
            value={params.max_tokens}
            onChange={(e) => updateParam('max_tokens', parseInt(e.target.value))}
            className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>Short</span>
            <span>Long</span>
          </div>
        </div>
      </div>

      {/* Advanced Parameters */}
      {showAdvanced && (
        <div className="space-y-4 pt-4 border-t">
          <div className="grid grid-cols-2 gap-4">
            {/* Top K */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm text-muted-foreground">Top K</label>
                <span className="text-sm font-medium">{params.top_k}</span>
              </div>
              <input
                type="range"
                min="1"
                max="100"
                step="1"
                value={params.top_k}
                onChange={(e) => updateParam('top_k', parseInt(e.target.value))}
                className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
              />
            </div>

            {/* Top P */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm text-muted-foreground">Top P</label>
                <span className="text-sm font-medium">{params.top_p}</span>
              </div>
              <input
                type="range"
                min="0.1"
                max="1.0"
                step="0.05"
                value={params.top_p}
                onChange={(e) => updateParam('top_p', parseFloat(e.target.value))}
                className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
              />
            </div>
          </div>

          {/* Additional Options */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm text-muted-foreground">Streaming</label>
                <p className="text-xs text-muted-foreground">
                  Enable real-time token streaming
                </p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={params.stream}
                  onChange={(e) => updateParam('stream', e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
              </label>
            </div>

            {/* Stop Sequences */}
            <div className="space-y-2">
              <label className="text-sm text-muted-foreground">Stop Sequences</label>
              <input
                type="text"
                placeholder="Enter stop sequences (comma-separated)"
                value={params.stop_sequences?.join(', ') || ''}
                onChange={(e) => {
                  const sequences = e.target.value
                    .split(',')
                    .map(s => s.trim())
                    .filter(s => s.length > 0);
                  updateParam('stop_sequences', sequences.length > 0 ? sequences : undefined);
                }}
                className="w-full px-3 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              />
            </div>
          </div>
        </div>
      )}

      {/* Performance Indicators */}
      <div className="flex items-center justify-between text-xs text-muted-foreground pt-2 border-t">
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-1">
            <Zap className="h-3 w-3" />
            <span>Speed: {(params.temperature || 0.7) < 0.5 ? 'Fast' : (params.temperature || 0.7) > 1.0 ? 'Slow' : 'Medium'}</span>
          </div>
          <div>
            Quality: {(params.temperature || 0.7) < 0.3 ? 'Deterministic' : (params.temperature || 0.7) > 0.8 ? 'Creative' : 'Balanced'}
          </div>
        </div>
        <Badge variant="secondary" className="text-xs">
          Est. {Math.ceil((params.max_tokens || 512) / 50)} sec
        </Badge>
      </div>
    </div>
  );
}