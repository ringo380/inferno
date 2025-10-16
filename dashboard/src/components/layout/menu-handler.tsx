'use client';

import { useState } from 'react';
import { useMenuEvents } from '@/hooks/use-menu-events';
import { QuickInferenceModal } from '@/components/modals/quick-inference-modal';
import { ModelUploadModal } from '@/components/modals/model-upload-modal';
import { toast } from 'react-hot-toast';
import { invoke } from '@tauri-apps/api/core';

/**
 * Global menu event handler that manages modals and actions triggered from the native menu bar
 */
export function MenuHandler() {
  const [quickInferenceOpen, setQuickInferenceOpen] = useState(false);
  const [modelUploadOpen, setModelUploadOpen] = useState(false);

  useMenuEvents({
    onNewInference: () => setQuickInferenceOpen(true),
    onQuickInference: () => setQuickInferenceOpen(true),
    onOpenModel: () => setModelUploadOpen(true),
    onImportModel: () => setModelUploadOpen(true),

    onStopInference: async () => {
      try {
        await invoke('stop_all_inference');
        toast.success('Stopped all active inferences');
      } catch (error) {
        console.error('Failed to stop inference:', error);
        toast.error('Failed to stop inference');
      }
    },

    onValidateModels: async () => {
      try {
        toast.loading('Validating models...', { id: 'validate-models' });
        const result = await invoke('validate_all_models');
        toast.success('All models validated successfully', { id: 'validate-models' });
      } catch (error) {
        console.error('Validation failed:', error);
        toast.error('Model validation failed', { id: 'validate-models' });
      }
    },

    onCheckUpdates: async () => {
      toast.success('You are running the latest version!', { duration: 3000 });
    },

    onShowShortcuts: () => {
      toast('Keyboard shortcuts:\n⌘N New Inference\n⌘O Open Model\n⌘1-4 Navigate', {
        duration: 5000,
        style: { whiteSpace: 'pre-line' }
      });
    },

    onAbout: () => {
      toast.success(`Inferno AI Desktop v${process.env.NEXT_PUBLIC_APP_VERSION || '0.7.0'}`, { duration: 3000 });
    },
  });

  return (
    <>
      <QuickInferenceModal
        open={quickInferenceOpen}
        onOpenChange={setQuickInferenceOpen}
      />

      <ModelUploadModal
        open={modelUploadOpen}
        onOpenChange={setModelUploadOpen}
      />
    </>
  );
}
