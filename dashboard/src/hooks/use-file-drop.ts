'use client';

import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'react-hot-toast';
import { useQueryClient } from '@tanstack/react-query';

interface FileDropEvent {
  paths: string[];
}

/**
 * Hook to handle file drop events for model installation
 * Automatically imports .gguf and .onnx files dropped onto the window
 */
export function useFileDrop() {
  const [isDropping, setIsDropping] = useState(false);
  const queryClient = useQueryClient();

  useEffect(() => {
    let unlistenHover: (() => void) | undefined;
    let unlistenDrop: (() => void) | undefined;
    let unlistenCancel: (() => void) | undefined;

    // Listen for file drop hover events
    listen<FileDropEvent>('tauri://file-drop-hover', (event) => {
      const paths = event.payload.paths;
      const hasModelFiles = paths.some(path =>
        path.endsWith('.gguf') || path.endsWith('.onnx')
      );

      if (hasModelFiles) {
        setIsDropping(true);
      }
    }).then((unlisten) => {
      unlistenHover = unlisten;
    });

    // Listen for file drop events
    listen<FileDropEvent>('tauri://file-drop', async (event) => {
      setIsDropping(false);

      const paths = event.payload.paths;
      const modelFiles = paths.filter(path =>
        path.endsWith('.gguf') || path.endsWith('.onnx')
      );

      if (modelFiles.length === 0) {
        toast.error('Please drop .gguf or .onnx model files', { duration: 3000 });
        return;
      }

      console.log(`📥 Dropped ${modelFiles.length} model file(s):`, modelFiles);

      // Import each model file
      for (const filePath of modelFiles) {
        const fileName = filePath.split('/').pop() || 'model';

        try {
          toast.loading(`Importing ${fileName}...`, { id: fileName });

          const targetPath = await invoke<string>('upload_model', {
            sourcePath: filePath,
            targetName: null, // Use original filename
          });

          toast.success(`✅ Imported ${fileName}`, { id: fileName, duration: 3000 });

          console.log(`✅ Model imported to: ${targetPath}`);
        } catch (error) {
          console.error(`Failed to import ${fileName}:`, error);
          toast.error(`Failed to import ${fileName}`, { id: fileName, duration: 4000 });
        }
      }

      // Refresh models list
      await queryClient.invalidateQueries({ queryKey: ['models'] });
      await queryClient.invalidateQueries({ queryKey: ['loaded-models'] });

      toast.success(`Imported ${modelFiles.length} model(s)`, { duration: 3000 });
    }).then((unlisten) => {
      unlistenDrop = unlisten;
    });

    // Listen for drop cancelled events
    listen('tauri://file-drop-cancelled', () => {
      setIsDropping(false);
    }).then((unlisten) => {
      unlistenCancel = unlisten;
    });

    return () => {
      unlistenHover?.();
      unlistenDrop?.();
      unlistenCancel?.();
    };
  }, [queryClient]);

  return { isDropping };
}
