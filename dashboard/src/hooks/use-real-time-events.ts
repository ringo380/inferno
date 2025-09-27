import { useEffect, useState, useCallback } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

// Event types that match the Rust InfernoEvent enum
export interface SystemMetricsEvent {
  cpu_usage: number;
  memory_usage: number;
  timestamp: string;
}

export interface ModelEvent {
  model_id: string;
  backend_id: string;
  timestamp: string;
}

export interface InferenceEvent {
  inference_id: string;
  model_id?: string;
  progress?: number;
  partial_response?: string;
  response?: string;
  latency_ms?: number;
  error?: string;
  timestamp: string;
}

export interface BatchJobEvent {
  job_id: string;
  name?: string;
  progress?: number;
  completed_tasks?: number;
  failed_tasks?: number;
  timestamp: string;
}

export interface SecurityEvent {
  event_id: string;
  event_type: string;
  severity: string;
  description: string;
  source_ip?: string;
  timestamp: string;
}

export interface ApiKeyEvent {
  key_id: string;
  name: string;
  permissions: string[];
  reason?: string;
  timestamp: string;
}

export interface ConnectionStatusEvent {
  status: 'Connected' | 'Disconnected' | 'Reconnecting' | { Error: { message: string } };
  timestamp: string;
}

export type InfernoEvent =
  | { type: 'SystemMetricsUpdated'; data: SystemMetricsEvent }
  | { type: 'ModelLoaded'; data: ModelEvent }
  | { type: 'ModelUnloaded'; data: ModelEvent }
  | { type: 'InferenceStarted'; data: InferenceEvent }
  | { type: 'InferenceProgress'; data: InferenceEvent }
  | { type: 'InferenceCompleted'; data: InferenceEvent }
  | { type: 'InferenceError'; data: InferenceEvent }
  | { type: 'BatchJobCreated'; data: BatchJobEvent }
  | { type: 'BatchJobStarted'; data: BatchJobEvent }
  | { type: 'BatchJobProgress'; data: BatchJobEvent }
  | { type: 'BatchJobCompleted'; data: BatchJobEvent }
  | { type: 'BatchJobFailed'; data: BatchJobEvent }
  | { type: 'SecurityEvent'; data: SecurityEvent }
  | { type: 'ApiKeyCreated'; data: ApiKeyEvent }
  | { type: 'ApiKeyRevoked'; data: ApiKeyEvent }
  | { type: 'ConnectionStatusChanged'; data: ConnectionStatusEvent };

// Hook for listening to all Inferno events
export function useRealTimeEvents() {
  const [events, setEvents] = useState<InfernoEvent[]>([]);
  const [latestEvent, setLatestEvent] = useState<InfernoEvent | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<'Connected' | 'Disconnected' | 'Connecting' | 'Reconnecting' | 'Error'>('Disconnected');

  useEffect(() => {
    let unlistenFn: UnlistenFn | null = null;

    const setupListener = async () => {
      try {
        // Listen to the main inferno_event channel
        unlistenFn = await listen('inferno_event', (event) => {
          const infernoEvent = event.payload as InfernoEvent;

          setLatestEvent(infernoEvent);
          setEvents(prev => [...prev.slice(-99), infernoEvent]); // Keep last 100 events

          // Update connection status based on events
          if (infernoEvent.type === 'ConnectionStatusChanged') {
            const status = infernoEvent.data.status;
            if (typeof status === 'string') {
              setConnectionStatus(status);
            } else {
              setConnectionStatus('Error');
            }
          } else {
            // If we're receiving events, we're connected
            setConnectionStatus('Connected');
          }
        });
      } catch (error) {
        console.error('Failed to setup event listener:', error);
        setConnectionStatus('Error');
      }
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  const clearEvents = useCallback(() => {
    setEvents([]);
    setLatestEvent(null);
  }, []);

  return {
    events,
    latestEvent,
    connectionStatus,
    clearEvents,
  };
}

// Hook for listening to specific event types
export function useRealTimeEventsByType<T extends InfernoEvent['type']>(eventType: T) {
  const [events, setEvents] = useState<Extract<InfernoEvent, { type: T }>[]>([]);
  const [latestEvent, setLatestEvent] = useState<Extract<InfernoEvent, { type: T }> | null>(null);

  useEffect(() => {
    let unlistenFn: UnlistenFn | null = null;

    const setupListener = async () => {
      try {
        unlistenFn = await listen('inferno_event', (event) => {
          const infernoEvent = event.payload as InfernoEvent;

          if (infernoEvent.type === eventType) {
            const typedEvent = infernoEvent as Extract<InfernoEvent, { type: T }>;
            setLatestEvent(typedEvent);
            setEvents(prev => [...prev.slice(-49), typedEvent]); // Keep last 50 events of this type
          }
        });
      } catch (error) {
        console.error(`Failed to setup listener for ${eventType}:`, error);
      }
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [eventType]);

  return {
    events,
    latestEvent,
  };
}

// Hook for listening to system metrics updates specifically
export function useRealTimeSystemMetrics() {
  const { latestEvent } = useRealTimeEventsByType('SystemMetricsUpdated');

  return {
    metrics: latestEvent?.data || null,
    timestamp: latestEvent?.data.timestamp || null,
  };
}

// Hook for listening to model events (load/unload)
export function useRealTimeModelEvents() {
  const [modelEvents, setModelEvents] = useState<ModelEvent[]>([]);

  useEffect(() => {
    let unlistenFn: UnlistenFn | null = null;

    const setupListener = async () => {
      try {
        unlistenFn = await listen('model_updated', (event) => {
          const infernoEvent = event.payload as InfernoEvent;

          if (infernoEvent.type === 'ModelLoaded' || infernoEvent.type === 'ModelUnloaded') {
            setModelEvents(prev => [...prev.slice(-19), infernoEvent.data]); // Keep last 20 model events
          }
        });
      } catch (error) {
        console.error('Failed to setup model event listener:', error);
      }
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  return {
    modelEvents,
  };
}

// Hook for listening to inference events specifically
export function useRealTimeInferenceEvents() {
  const [inferenceEvents, setInferenceEvents] = useState<InferenceEvent[]>([]);
  const [activeInferences, setActiveInferences] = useState<Map<string, InferenceEvent>>(new Map());

  useEffect(() => {
    let unlistenFn: UnlistenFn | null = null;

    const setupListener = async () => {
      try {
        unlistenFn = await listen('inference_updated', (event) => {
          const infernoEvent = event.payload as InfernoEvent;

          if (infernoEvent.type.startsWith('Inference')) {
            const inferenceData = infernoEvent.data as InferenceEvent;
            setInferenceEvents(prev => [...prev.slice(-29), inferenceData]); // Keep last 30 inference events

            // Track active inferences
            setActiveInferences(prev => {
              const newMap = new Map(prev);
              if (infernoEvent.type === 'InferenceStarted' || infernoEvent.type === 'InferenceProgress') {
                newMap.set(inferenceData.inference_id, inferenceData);
              } else if (infernoEvent.type === 'InferenceCompleted' || infernoEvent.type === 'InferenceError') {
                newMap.delete(inferenceData.inference_id);
              }
              return newMap;
            });
          }
        });
      } catch (error) {
        console.error('Failed to setup inference event listener:', error);
      }
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  return {
    inferenceEvents,
    activeInferences: Array.from(activeInferences.values()),
    activeInferenceCount: activeInferences.size,
  };
}

// Hook for listening to security events
export function useRealTimeSecurityEvents() {
  const [securityEvents, setSecurityEvents] = useState<SecurityEvent[]>([]);
  const [apiKeyEvents, setApiKeyEvents] = useState<ApiKeyEvent[]>([]);

  useEffect(() => {
    let unlistenFn: UnlistenFn | null = null;

    const setupListener = async () => {
      try {
        unlistenFn = await listen('security_updated', (event) => {
          const infernoEvent = event.payload as InfernoEvent;

          if (infernoEvent.type === 'SecurityEvent') {
            setSecurityEvents(prev => [...prev.slice(-19), infernoEvent.data]); // Keep last 20 security events
          } else if (infernoEvent.type === 'ApiKeyCreated' || infernoEvent.type === 'ApiKeyRevoked') {
            setApiKeyEvents(prev => [...prev.slice(-19), infernoEvent.data]); // Keep last 20 API key events
          }
        });
      } catch (error) {
        console.error('Failed to setup security event listener:', error);
      }
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  return {
    securityEvents,
    apiKeyEvents,
  };
}

// Hook for listening to batch job events
export function useRealTimeBatchJobEvents() {
  const [batchJobEvents, setBatchJobEvents] = useState<BatchJobEvent[]>([]);
  const [activeBatchJobs, setActiveBatchJobs] = useState<Map<string, BatchJobEvent>>(new Map());

  useEffect(() => {
    let unlistenFn: UnlistenFn | null = null;

    const setupListener = async () => {
      try {
        unlistenFn = await listen('batch_job_updated', (event) => {
          const infernoEvent = event.payload as InfernoEvent;

          if (infernoEvent.type.startsWith('BatchJob')) {
            const batchJobData = infernoEvent.data as BatchJobEvent;
            setBatchJobEvents(prev => [...prev.slice(-19), batchJobData]); // Keep last 20 batch job events

            // Track active batch jobs
            setActiveBatchJobs(prev => {
              const newMap = new Map(prev);
              if (infernoEvent.type === 'BatchJobCreated' ||
                  infernoEvent.type === 'BatchJobStarted' ||
                  infernoEvent.type === 'BatchJobProgress') {
                newMap.set(batchJobData.job_id, batchJobData);
              } else if (infernoEvent.type === 'BatchJobCompleted' || infernoEvent.type === 'BatchJobFailed') {
                newMap.delete(batchJobData.job_id);
              }
              return newMap;
            });
          }
        });
      } catch (error) {
        console.error('Failed to setup batch job event listener:', error);
      }
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  return {
    batchJobEvents,
    activeBatchJobs: Array.from(activeBatchJobs.values()),
    activeBatchJobCount: activeBatchJobs.size,
  };
}