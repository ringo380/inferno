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
  progress: number;
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

interface StreamingProgressEvent {
  inference_id: string;
  progress: number;
  partial_response?: string;
}

export function useStreamingInference() {
  const [state, setState] = useState<StreamingInferenceState>({
    isStreaming: false,
    currentText: '',
    isComplete: false,
    error: null,
    inferenceId: null,
    progress: 0,
  });

  const notifyNative = useCallback((title: string, body: string) => {
    void tauriApi
      .sendNativeNotification({ title, body })
      .catch((error) => console.warn('Native notification failed:', error));
  }, []);

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
        progress: 0,
      });

      const inferenceId = await tauriApi.inferStream(backendId, prompt, params);

      setState(prev => ({
        ...prev,
        inferenceId,
        progress: 0,
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
      progress: 0,
    });
  }, []);

  // Set up event listeners
  useEffect(() => {
    let unlistenToken: (() => void) | null = null;
    let unlistenComplete: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;
    let unlistenStart: (() => void) | null = null;
    let unlistenProgress: (() => void) | null = null;

    const setupListeners = async () => {
      // Listen for streaming start
      unlistenStart = await listen('inference_start', (event) => {
        const inferenceId = event.payload as string;
        setState(prev => ({
          ...prev,
          inferenceId,
          isStreaming: true,
          progress: 0,
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
              progress: 1,
            };
          }
          return prev;
        });

        notifyNative('Inference Complete', 'Streaming response finished successfully.');
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
              progress: prev.progress > 0 ? prev.progress : 0,
            };
          }
          return prev;
        });

        notifyNative('Inference Failed', errorEvent.error);
      });

      // Listen for progress updates
      unlistenProgress = await listen('inference_progress', (event) => {
        const progressEvent = event.payload as StreamingProgressEvent;
        setState(prev => {
          if (prev.inferenceId === progressEvent.inference_id) {
            const progressValue = Math.min(Math.max(progressEvent.progress ?? 0, 0), 1);
            return {
              ...prev,
              progress: progressValue,
              currentText: progressEvent.partial_response ?? prev.currentText,
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
      if (unlistenProgress) unlistenProgress();
    };
  }, [notifyNative]);

  return {
    ...state,
    startStreaming,
    stopStreaming,
  };
}
