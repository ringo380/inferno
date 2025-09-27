'use client';

import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { tauriApi } from '@/lib/tauri-api';
import { InferenceParams } from '@/types/inferno';

interface StreamingInferenceState {
  isStreaming: boolean;
  currentText: string;
  isComplete: boolean;
  error: string | null;
  inferenceId: string | null;
}

interface StreamingTokenEvent {
  inference_id: string;
  token: string;
}

interface StreamingCompleteEvent {
  inference_id: string;
  response: string;
}

interface StreamingErrorEvent {
  inference_id: string;
  error: string;
}

export function useStreamingInference() {
  const [state, setState] = useState<StreamingInferenceState>({
    isStreaming: false,
    currentText: '',
    isComplete: false,
    error: null,
    inferenceId: null,
  });

  // Start streaming inference
  const startStreaming = useCallback(async (
    backendId: string,
    prompt: string,
    params: InferenceParams
  ) => {
    try {
      setState({
        isStreaming: true,
        currentText: '',
        isComplete: false,
        error: null,
        inferenceId: null,
      });

      const inferenceId = await tauriApi.inferStream(backendId, prompt, params);

      setState(prev => ({
        ...prev,
        inferenceId,
      }));

      return inferenceId;
    } catch (error) {
      setState(prev => ({
        ...prev,
        isStreaming: false,
        error: String(error),
      }));
      throw error;
    }
  }, []);

  // Stop streaming (reset state)
  const stopStreaming = useCallback(() => {
    setState({
      isStreaming: false,
      currentText: '',
      isComplete: false,
      error: null,
      inferenceId: null,
    });
  }, []);

  // Set up event listeners
  useEffect(() => {
    let unlistenToken: (() => void) | null = null;
    let unlistenComplete: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;
    let unlistenStart: (() => void) | null = null;

    const setupListeners = async () => {
      // Listen for streaming start
      unlistenStart = await listen('inference_start', (event) => {
        const inferenceId = event.payload as string;
        setState(prev => ({
          ...prev,
          inferenceId,
          isStreaming: true,
        }));
      });

      // Listen for streaming tokens
      unlistenToken = await listen('inference_token', (event) => {
        const tokenEvent = event.payload as StreamingTokenEvent;
        setState(prev => {
          // Only update if this is the current inference session
          if (prev.inferenceId === tokenEvent.inference_id) {
            return {
              ...prev,
              currentText: prev.currentText + tokenEvent.token,
            };
          }
          return prev;
        });
      });

      // Listen for completion
      unlistenComplete = await listen('inference_complete', (event) => {
        const completeEvent = event.payload as StreamingCompleteEvent;
        setState(prev => {
          // Only update if this is the current inference session
          if (prev.inferenceId === completeEvent.inference_id) {
            return {
              ...prev,
              isStreaming: false,
              isComplete: true,
              currentText: completeEvent.response, // Use final response
            };
          }
          return prev;
        });
      });

      // Listen for errors
      unlistenError = await listen('inference_error', (event) => {
        const errorEvent = event.payload as StreamingErrorEvent;
        setState(prev => {
          // Only update if this is the current inference session
          if (prev.inferenceId === errorEvent.inference_id) {
            return {
              ...prev,
              isStreaming: false,
              error: errorEvent.error,
            };
          }
          return prev;
        });
      });
    };

    setupListeners();

    // Cleanup listeners on unmount
    return () => {
      if (unlistenStart) unlistenStart();
      if (unlistenToken) unlistenToken();
      if (unlistenComplete) unlistenComplete();
      if (unlistenError) unlistenError();
    };
  }, []);

  return {
    ...state,
    startStreaming,
    stopStreaming,
  };
}