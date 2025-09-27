'use client';

import React from 'react';
import { useRealTimeEvents } from '@/hooks/use-real-time-events';
import { cn } from '@/lib/utils';

interface ConnectionStatusProps {
  className?: string;
  showText?: boolean;
  compact?: boolean;
}

export function ConnectionStatus({ className, showText = true, compact = false }: ConnectionStatusProps) {
  const { connectionStatus, latestEvent } = useRealTimeEvents();

  const getStatusColor = (status: typeof connectionStatus) => {
    switch (status) {
      case 'Connected':
        return 'bg-green-500';
      case 'Connecting':
      case 'Reconnecting':
        return 'bg-yellow-500';
      case 'Disconnected':
        return 'bg-gray-500';
      case 'Error':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  const getStatusText = (status: typeof connectionStatus) => {
    switch (status) {
      case 'Connected':
        return 'Connected';
      case 'Connecting':
        return 'Connecting...';
      case 'Reconnecting':
        return 'Reconnecting...';
      case 'Disconnected':
        return 'Disconnected';
      case 'Error':
        return 'Connection Error';
      default:
        return 'Unknown';
    }
  };

  const getStatusIcon = (status: typeof connectionStatus) => {
    switch (status) {
      case 'Connected':
        return '●';
      case 'Connecting':
      case 'Reconnecting':
        return '◐';
      case 'Disconnected':
        return '○';
      case 'Error':
        return '⚠';
      default:
        return '?';
    }
  };

  if (compact) {
    return (
      <div className={cn("flex items-center gap-1.5", className)}>
        <div
          className={cn(
            "w-2 h-2 rounded-full transition-colors duration-200",
            getStatusColor(connectionStatus)
          )}
          title={`Real-time status: ${getStatusText(connectionStatus)}`}
        />
        {showText && (
          <span className="text-xs text-gray-600 dark:text-gray-400">
            {getStatusText(connectionStatus)}
          </span>
        )}
      </div>
    );
  }

  return (
    <div className={cn("flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gray-50 dark:bg-gray-800", className)}>
      <div className="flex items-center gap-1.5">
        <div
          className={cn(
            "w-3 h-3 rounded-full transition-colors duration-200",
            getStatusColor(connectionStatus),
            connectionStatus === 'Connecting' || connectionStatus === 'Reconnecting' ? 'animate-pulse' : ''
          )}
        />
        {showText && (
          <div className="flex flex-col">
            <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
              {getStatusText(connectionStatus)}
            </span>
            {latestEvent && connectionStatus === 'Connected' && (
              <span className="text-xs text-gray-500 dark:text-gray-400">
                Last: {new Date(latestEvent.data.timestamp).toLocaleTimeString()}
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// Alternative compact version for headers/navigation
export function ConnectionStatusIndicator({ className }: { className?: string }) {
  const { connectionStatus } = useRealTimeEvents();

  return (
    <div
      className={cn(
        "flex items-center gap-1.5 px-2 py-1 rounded-md text-xs font-medium transition-colors duration-200",
        connectionStatus === 'Connected'
          ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
          : connectionStatus === 'Error'
          ? "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
          : "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400",
        className
      )}
      title={`Real-time connection: ${connectionStatus}`}
    >
      <span className="text-xs">
        {getStatusIcon(connectionStatus)}
      </span>
      <span className="hidden sm:inline">
        {connectionStatus}
      </span>
    </div>
  );
}

function getStatusIcon(status: string) {
  switch (status) {
    case 'Connected':
      return '●';
    case 'Connecting':
    case 'Reconnecting':
      return '◐';
    case 'Disconnected':
      return '○';
    case 'Error':
      return '⚠';
    default:
      return '?';
  }
}

// Real-time event stream component for debugging
export function EventStreamDebugger() {
  const { events, connectionStatus, clearEvents } = useRealTimeEvents();

  if (process.env.NODE_ENV !== 'development') {
    return null;
  }

  return (
    <div className="fixed bottom-4 right-4 w-80 max-h-96 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg z-50">
      <div className="flex items-center justify-between p-3 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-medium">Event Stream</h3>
          <ConnectionStatus compact showText={false} />
        </div>
        <button
          onClick={clearEvents}
          className="text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
        >
          Clear
        </button>
      </div>
      <div className="max-h-80 overflow-y-auto p-3 space-y-2">
        {events.length === 0 ? (
          <div className="text-xs text-gray-500 dark:text-gray-400 text-center py-4">
            No events received yet...
          </div>
        ) : (
          events.slice(-10).map((event, index) => (
            <div key={index} className="text-xs space-y-1">
              <div className="flex items-center justify-between">
                <span className="font-medium text-blue-600 dark:text-blue-400">
                  {event.type}
                </span>
                <span className="text-gray-500 dark:text-gray-400">
                  {new Date(event.data.timestamp).toLocaleTimeString()}
                </span>
              </div>
              <div className="text-gray-600 dark:text-gray-300 bg-gray-50 dark:bg-gray-700 rounded p-1 overflow-hidden">
                {JSON.stringify(event.data, null, 2).substring(0, 100)}
                {JSON.stringify(event.data).length > 100 && '...'}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}