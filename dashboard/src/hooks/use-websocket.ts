'use client';

import { useEffect, useRef, useState } from 'react';
import { io, Socket } from 'socket.io-client';
import { WebSocketMessage } from '@/types/inferno';

interface UseWebSocketOptions {
  enabled?: boolean;
  reconnect?: boolean;
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (error: Error) => void;
}

export function useWebSocket(
  url: string = process.env.INFERNO_WS_URL || 'ws://localhost:8080',
  options: UseWebSocketOptions = {}
) {
  const {
    enabled = true,
    reconnect = true,
    onConnect,
    onDisconnect,
    onError,
  } = options;

  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const socketRef = useRef<Socket | null>(null);
  const listenersRef = useRef<Map<string, Set<(data: any) => void>>>(new Map());

  useEffect(() => {
    if (!enabled) return;

    // Create socket connection
    const socket = io(url, {
      autoConnect: true,
      reconnection: reconnect,
      reconnectionAttempts: 5,
      reconnectionDelay: 1000,
      timeout: 10000,
    });

    socketRef.current = socket;

    // Connection event handlers
    socket.on('connect', () => {
      setIsConnected(true);
      setError(null);
      onConnect?.();
    });

    socket.on('disconnect', () => {
      setIsConnected(false);
      onDisconnect?.();
    });

    socket.on('connect_error', (err) => {
      const error = new Error(`WebSocket connection failed: ${err.message}`);
      setError(error);
      onError?.(error);
    });

    // Generic message handler
    socket.onAny((event: string, data: any) => {
      const listeners = listenersRef.current.get(event);
      if (listeners) {
        listeners.forEach(listener => listener(data));
      }
    });

    return () => {
      socket.disconnect();
      socketRef.current = null;
      setIsConnected(false);
    };
  }, [url, enabled, reconnect, onConnect, onDisconnect, onError]);

  const subscribe = (event: string, callback: (data: any) => void) => {
    if (!listenersRef.current.has(event)) {
      listenersRef.current.set(event, new Set());
    }
    listenersRef.current.get(event)!.add(callback);

    // Return unsubscribe function
    return () => {
      const listeners = listenersRef.current.get(event);
      if (listeners) {
        listeners.delete(callback);
        if (listeners.size === 0) {
          listenersRef.current.delete(event);
        }
      }
    };
  };

  const emit = (event: string, data?: any) => {
    if (socketRef.current?.connected) {
      socketRef.current.emit(event, data);
    }
  };

  const disconnect = () => {
    socketRef.current?.disconnect();
  };

  const reconnectSocket = () => {
    socketRef.current?.connect();
  };

  return {
    isConnected,
    error,
    subscribe,
    emit,
    disconnect,
    reconnect: reconnectSocket,
  };
}

// Specialized hooks for specific data types
export function useSystemMetrics() {
  const [metrics, setMetrics] = useState<any>(null);
  const { subscribe, isConnected } = useWebSocket();

  useEffect(() => {
    const unsubscribe = subscribe('metrics', (data: any) => {
      setMetrics(data);
    });

    return unsubscribe;
  }, [subscribe]);

  return { metrics, isConnected };
}

export function useInferenceStream(onToken?: (token: string) => void) {
  const { subscribe, emit, isConnected } = useWebSocket();

  useEffect(() => {
    if (!onToken) return;

    const unsubscribe = subscribe('inference_stream', (data: { token: string; done: boolean }) => {
      if (data.token && !data.done) {
        onToken(data.token);
      }
    });

    return unsubscribe;
  }, [subscribe, onToken]);

  const startInference = (request: any) => {
    emit('start_inference', request);
  };

  const stopInference = () => {
    emit('stop_inference');
  };

  return { startInference, stopInference, isConnected };
}

export function useJobUpdates() {
  const [jobUpdates, setJobUpdates] = useState<any[]>([]);
  const { subscribe, isConnected } = useWebSocket();

  useEffect(() => {
    const unsubscribe = subscribe('job_update', (data: any) => {
      setJobUpdates(prev => [data, ...prev.slice(0, 99)]); // Keep last 100 updates
    });

    return unsubscribe;
  }, [subscribe]);

  return { jobUpdates, isConnected };
}

export function useSecurityAlerts() {
  const [alerts, setAlerts] = useState<any[]>([]);
  const { subscribe, isConnected } = useWebSocket();

  useEffect(() => {
    const unsubscribe = subscribe('security_alert', (data: any) => {
      setAlerts(prev => [data, ...prev.slice(0, 49)]); // Keep last 50 alerts
    });

    return unsubscribe;
  }, [subscribe]);

  return { alerts, isConnected };
}

export function useSystemStatus() {
  const [status, setStatus] = useState<string>('unknown');
  const { subscribe, isConnected } = useWebSocket();

  useEffect(() => {
    const unsubscribe = subscribe('system_status', (data: { status: string }) => {
      setStatus(data.status);
    });

    return unsubscribe;
  }, [subscribe]);

  return { status, isConnected };
}