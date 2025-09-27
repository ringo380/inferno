'use client';

import React from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Activity,
  Pause,
  Play,
  RefreshCw,
  Settings,
  Wifi,
  WifiOff,
  Clock,
  Zap,
  Snail
} from 'lucide-react';
import { useRealTimeUpdates } from '@/components/providers/real-time-updates';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

export function RealTimeStatus() {
  const {
    settings,
    updateSettings,
    toggleRealTime,
    forceRefresh,
    lastUpdateTime,
    isUpdating
  } = useRealTimeUpdates();

  const formatLastUpdate = (date: Date | null) => {
    if (!date) return 'Never';
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    return `${Math.floor(diff / 3600000)}h ago`;
  };

  const getModeIcon = (mode: string) => {
    switch (mode) {
      case 'live': return <Zap className="h-3 w-3" />;
      case 'normal': return <Activity className="h-3 w-3" />;
      case 'minimal': return <Snail className="h-3 w-3" />;
      default: return <Activity className="h-3 w-3" />;
    }
  };

  const getModeColor = (mode: string) => {
    switch (mode) {
      case 'live': return 'bg-green-500';
      case 'normal': return 'bg-blue-500';
      case 'minimal': return 'bg-orange-500';
      default: return 'bg-gray-500';
    }
  };

  return (
    <div className="flex items-center space-x-2 text-sm">
      {/* Connection Status */}
      <div className="flex items-center space-x-1">
        {settings.enabled ? (
          <Wifi className="h-4 w-4 text-green-500" />
        ) : (
          <WifiOff className="h-4 w-4 text-gray-400" />
        )}
        <span className="text-xs text-muted-foreground">
          {settings.enabled ? 'Live' : 'Offline'}
        </span>
      </div>

      {/* Update Indicator */}
      {isUpdating && (
        <div className="flex items-center space-x-1">
          <RefreshCw className="h-3 w-3 animate-spin text-blue-500" />
          <span className="text-xs text-blue-500">Updating...</span>
        </div>
      )}

      {/* Last Update Time */}
      {!isUpdating && lastUpdateTime && (
        <div className="flex items-center space-x-1">
          <Clock className="h-3 w-3 text-muted-foreground" />
          <span className="text-xs text-muted-foreground">
            {formatLastUpdate(lastUpdateTime)}
          </span>
        </div>
      )}

      {/* Mode Badge */}
      <Badge variant="secondary" className="flex items-center space-x-1 text-xs">
        {getModeIcon(settings.mode)}
        <span className="capitalize">{settings.mode}</span>
      </Badge>

      {/* Controls Menu */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="sm" className="h-8 px-2">
            <Settings className="h-3 w-3" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          <DropdownMenuLabel>Real-time Updates</DropdownMenuLabel>
          <DropdownMenuSeparator />

          <DropdownMenuItem onClick={toggleRealTime}>
            {settings.enabled ? (
              <>
                <Pause className="h-4 w-4 mr-2" />
                Pause Updates
              </>
            ) : (
              <>
                <Play className="h-4 w-4 mr-2" />
                Resume Updates
              </>
            )}
          </DropdownMenuItem>

          <DropdownMenuItem onClick={forceRefresh}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh All Data
          </DropdownMenuItem>

          <DropdownMenuSeparator />
          <DropdownMenuLabel>Update Mode</DropdownMenuLabel>

          <DropdownMenuItem
            onClick={() => updateSettings({ mode: 'live' })}
            className={settings.mode === 'live' ? 'bg-accent' : ''}
          >
            <div className="flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full bg-green-500`} />
              <div>
                <div className="font-medium">Live (1-2s)</div>
                <div className="text-xs text-muted-foreground">Maximum responsiveness</div>
              </div>
            </div>
          </DropdownMenuItem>

          <DropdownMenuItem
            onClick={() => updateSettings({ mode: 'normal' })}
            className={settings.mode === 'normal' ? 'bg-accent' : ''}
          >
            <div className="flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full bg-blue-500`} />
              <div>
                <div className="font-medium">Normal (2-5s)</div>
                <div className="text-xs text-muted-foreground">Balanced performance</div>
              </div>
            </div>
          </DropdownMenuItem>

          <DropdownMenuItem
            onClick={() => updateSettings({ mode: 'minimal' })}
            className={settings.mode === 'minimal' ? 'bg-accent' : ''}
          >
            <div className="flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full bg-orange-500`} />
              <div>
                <div className="font-medium">Minimal (5-30s)</div>
                <div className="text-xs text-muted-foreground">Battery saving</div>
              </div>
            </div>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}