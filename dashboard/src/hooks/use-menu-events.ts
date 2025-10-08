'use client';

import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useRouter } from 'next/navigation';
import { toast } from 'react-hot-toast';

/**
 * Hook to listen for menu and tray events from the native macOS menu bar
 */
export function useMenuEvents(handlers?: {
  onOpenPreferences?: () => void;
  onNewInference?: () => void;
  onOpenModel?: () => void;
  onImportModel?: () => void;
  onExportResults?: () => void;
  onModelInfo?: () => void;
  onValidateModels?: () => void;
  onQuickInference?: () => void;
  onBatchInference?: () => void;
  onStopInference?: () => void;
  onOpenDocs?: () => void;
  onShowShortcuts?: () => void;
  onReportIssue?: () => void;
  onCheckUpdates?: () => void;
  onAbout?: () => void;
}) {
  const router = useRouter();

  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    // Menu: Preferences
    listen('menu://open-preferences', () => {
      console.log('📋 Menu: Open Preferences');
      handlers?.onOpenPreferences?.() || router.push('/settings');
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: New Inference
    listen('menu://new-inference', () => {
      console.log('⚡ Menu: New Inference');
      handlers?.onNewInference?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Open Model
    listen('menu://open-model', () => {
      console.log('📂 Menu: Open Model');
      handlers?.onOpenModel?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Import Model
    listen('menu://import-model', () => {
      console.log('📥 Menu: Import Model');
      handlers?.onImportModel?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Export Results
    listen('menu://export-results', () => {
      console.log('📤 Menu: Export Results');
      handlers?.onExportResults?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Model Info
    listen('menu://model-info', () => {
      console.log('ℹ️ Menu: Model Info');
      handlers?.onModelInfo?.() || router.push('/models');
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Validate Models
    listen('menu://validate-models', () => {
      console.log('✅ Menu: Validate Models');
      handlers?.onValidateModels?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Quick Inference
    listen('menu://quick-inference', () => {
      console.log('⚡ Menu: Quick Inference');
      handlers?.onQuickInference?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Batch Inference
    listen('menu://batch-inference', () => {
      console.log('📦 Menu: Batch Inference');
      handlers?.onBatchInference?.() || router.push('/batch-jobs');
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Stop Inference
    listen('menu://stop-inference', () => {
      console.log('🛑 Menu: Stop Inference');
      handlers?.onStopInference?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Navigation
    listen<{ target: string }>('menu://navigate', (event) => {
      const target = event.payload.target;
      console.log(`🧭 Menu: Navigate to ${target}`);
      router.push(`/${target}`);
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Open Documentation
    listen('menu://open-docs', () => {
      console.log('📚 Menu: Open Documentation');
      handlers?.onOpenDocs?.() || window.open('https://github.com/ringo380/inferno#readme', '_blank');
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Show Shortcuts
    listen('menu://show-shortcuts', () => {
      console.log('⌨️ Menu: Show Shortcuts');
      handlers?.onShowShortcuts?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Report Issue
    listen('menu://report-issue', () => {
      console.log('🐛 Menu: Report Issue');
      handlers?.onReportIssue?.() || window.open('https://github.com/ringo380/inferno/issues/new', '_blank');
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: Check Updates
    listen('menu://check-updates', () => {
      console.log('🔄 Menu: Check Updates');
      handlers?.onCheckUpdates?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Menu: About
    listen('menu://about', () => {
      console.log('ℹ️ Menu: About');
      handlers?.onAbout?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // Tray: Open Dashboard
    listen('tray://open-dashboard', () => {
      console.log('🏠 Tray: Open Dashboard');
      router.push('/');
    }).then((unlisten) => unlisteners.push(unlisten));

    // Tray: Open Models
    listen('tray://open-models', () => {
      console.log('📦 Tray: Open Models');
      router.push('/models');
    }).then((unlisten) => unlisteners.push(unlisten));

    // Tray: Quick Inference
    listen('tray://quick-inference', () => {
      console.log('⚡ Tray: Quick Inference');
      handlers?.onQuickInference?.();
    }).then((unlisten) => unlisteners.push(unlisten));

    // System: Appearance Changed
    listen<string>('appearance-changed', (event) => {
      const appearance = event.payload;
      console.log(`🎨 System appearance changed: ${appearance}`);
      toast.success(`Switched to ${appearance} mode`, { duration: 2000 });
    }).then((unlisten) => unlisteners.push(unlisten));

    // Cleanup
    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [router, handlers]);
}
