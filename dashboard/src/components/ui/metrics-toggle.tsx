'use client';

import { useState, useEffect } from 'react';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Activity, Server } from 'lucide-react';

export type MetricsScope = 'inferno' | 'system';

interface MetricsToggleProps {
  value: MetricsScope;
  onChange: (scope: MetricsScope) => void;
  className?: string;
}

export function MetricsToggle({ value, onChange, className }: MetricsToggleProps) {
  const isSystemMode = value === 'system';

  return (
    <div className={`flex items-center space-x-3 ${className}`}>
      <div className="flex items-center space-x-2">
        <Activity className={`h-4 w-4 ${!isSystemMode ? 'text-primary' : 'text-muted-foreground'}`} />
        <Label
          htmlFor="metrics-toggle"
          className={`text-sm font-medium cursor-pointer ${!isSystemMode ? 'text-foreground' : 'text-muted-foreground'}`}
        >
          Inferno
        </Label>
      </div>

      <Switch
        id="metrics-toggle"
        checked={isSystemMode}
        onCheckedChange={(checked) => onChange(checked ? 'system' : 'inferno')}
        className="data-[state=checked]:bg-primary"
      />

      <div className="flex items-center space-x-2">
        <Server className={`h-4 w-4 ${isSystemMode ? 'text-primary' : 'text-muted-foreground'}`} />
        <Label
          htmlFor="metrics-toggle"
          className={`text-sm font-medium cursor-pointer ${isSystemMode ? 'text-foreground' : 'text-muted-foreground'}`}
        >
          System
        </Label>
      </div>
    </div>
  );
}

// Hook for managing metrics scope with localStorage persistence
export function useMetricsScope() {
  const [metricsScope, setMetricsScope] = useState<MetricsScope>('inferno');

  // Load from localStorage on mount
  useEffect(() => {
    const saved = localStorage.getItem('metrics-scope');
    if (saved === 'system' || saved === 'inferno') {
      setMetricsScope(saved);
    }
  }, []);

  // Save to localStorage when changed
  const updateMetricsScope = (scope: MetricsScope) => {
    setMetricsScope(scope);
    localStorage.setItem('metrics-scope', scope);
  };

  return {
    metricsScope,
    setMetricsScope: updateMetricsScope,
    isInfernoMode: metricsScope === 'inferno',
    isSystemMode: metricsScope === 'system',
  };
}