'use client';

import { useRouter } from 'next/navigation';
import { useEffect } from 'react';
import { toast } from 'react-hot-toast';
import { tauriApi } from '@/lib/tauri-api';
import type { Event } from '@tauri-apps/api/event';

const DOCS_URL = 'https://github.com/inferno-ai/inferno/tree/main/docs';
const ISSUES_URL = 'https://github.com/inferno-ai/inferno/issues/new/choose';
const RELEASES_URL = 'https://github.com/inferno-ai/inferno/releases';

const isTauriRuntime = () =>
  typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__);

export function DesktopBridge() {
  const router = useRouter();

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    const unlistenFns: Array<() => void | Promise<void>> = [];

    const safePush = (path: string) => {
      try {
        router.push(path);
      } catch (error) {
        console.error('DesktopBridge navigation failed:', error);
      }
    };

    const dispatchWindowEvent = (eventName: string) => {
      if (typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent(eventName));
      }
    };

    const openExternal = async (url: string) => {
      try {
        if (typeof window !== 'undefined') {
          window.open(url, '_blank', 'noopener,noreferrer');
        }
      } catch (error) {
        console.error('Failed to open external url:', error);
        toast.error('Unable to open external link');
      }
    };

    const handleImportModel = async () => {
      try {
        const sourcePath = await tauriApi.openFileDialog();
        if (!sourcePath) {
          return;
        }

        const targetName = sourcePath.split(/[/\\]/).pop();
        await tauriApi.uploadModel(sourcePath, targetName ?? undefined);
        toast.success('Model import started');
        safePush('/models');
      } catch (error) {
        console.error('Model import failed:', error);
        toast.error('Failed to import model');
      }
    };

    const registerListeners = async () => {
      const { listen } = await import('@tauri-apps/api/event');

      const addListener = async <T,>(
        event: string,
        handler: (event: Event<T>) => void
      ) => {
        const unlisten = await listen<T>(event, handler);
        unlistenFns.push(unlisten);
      };

      const routeEvents: Record<string, string> = {
        'menu://open-preferences': '/settings',
        'menu://open-model': '/models',
        'menu://model-info': '/models',
        'menu://validate-models': '/models',
        'menu://batch-inference': '/batch',
        'tray://open-dashboard': '/',
        'tray://open-models': '/models',
      };

      for (const [eventName, path] of Object.entries(routeEvents)) {
        await addListener(eventName, () => safePush(path));
      }

      await addListener('menu://new-inference', () => {
        safePush('/inference');
        dispatchWindowEvent('inferno:quick-inference');
      });

      await addListener('menu://quick-inference', () => {
        safePush('/inference');
        dispatchWindowEvent('inferno:quick-inference');
      });

      await addListener('menu://stop-inference', () => {
        dispatchWindowEvent('inferno:stop-inference');
      });

      await addListener('menu://import-model', () => {
        void handleImportModel();
      });

      await addListener('menu://export-results', () => {
        toast('Export results is coming soon');
      });

      await addListener('menu://open-docs', () => {
        void openExternal(DOCS_URL);
      });

      await addListener('menu://report-issue', () => {
        void openExternal(ISSUES_URL);
      });

      await addListener('menu://check-updates', () => {
        void openExternal(RELEASES_URL);
      });

      await addListener('menu://show-shortcuts', () => {
        dispatchWindowEvent('inferno:toggle-shortcuts');
      });

      await addListener('menu://about', () => {
        dispatchWindowEvent('inferno:show-about');
      });

      await addListener('tray://show', () => {
        dispatchWindowEvent('inferno:tray-show');
      });

      await addListener('tray://hide', () => {
        dispatchWindowEvent('inferno:tray-hide');
      });

      await addListener<{ target?: string }>('menu://navigate', (event) => {
        const target = event.payload?.target;
        if (!target) return;

        const navigationMap: Record<string, string> = {
          dashboard: '/',
          models: '/models',
          inference: '/inference',
          metrics: '/monitoring',
          monitoring: '/monitoring',
          batch: '/batch',
          settings: '/settings',
        };

        const path = navigationMap[target];
        if (path) {
          safePush(path);
        }
      });

      await addListener('tray://quick-inference', () => {
        safePush('/inference');
        dispatchWindowEvent('inferno:quick-inference');
      });

      await addListener<{ type?: string; data?: Record<string, any> }>('batch_job_updated', (event) => {
        const payload = event.payload;
        const eventType = payload?.type;
        const jobId = payload?.data?.job_id ?? 'Batch job';

        switch (eventType) {
          case 'BatchJobCompleted':
            void tauriApi.sendNativeNotification({
              title: 'Batch Job Completed',
              body: `${jobId} finished successfully.`,
            });
            break;
          case 'BatchJobFailed':
            void tauriApi.sendNativeNotification({
              title: 'Batch Job Failed',
              body: payload?.data?.error ? `${jobId}: ${payload.data.error}` : `${jobId} encountered an error.`,
            });
            break;
          default:
            break;
        }
      });
    };

    registerListeners().catch(error => {
      console.error('Failed to register desktop bridge listeners:', error);
    });

    return () => {
      if (unlistenFns.length > 0) {
        for (const unlisten of unlistenFns) {
          void unlisten();
        }
      }
    };
  }, [router]);

  return null;
}
