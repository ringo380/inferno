'use client';

import { useEffect } from 'react';
import { useTheme } from 'next-themes';
import { useVibrancy, useSystemAppearance } from '@/hooks/use-vibrancy';

/**
 * Client component wrapper that applies macOS window vibrancy effects
 * and syncs theme with system appearance
 */
export function VibrancyWrapper() {
  const { theme, setTheme } = useTheme();

  // Apply vibrancy effect
  useVibrancy('contentBackground');

  // Sync system appearance with theme on startup
  useEffect(() => {
    const syncSystemAppearance = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const appearance = await invoke<string>('get_system_appearance');

        if (appearance === 'dark' && theme !== 'dark') {
          setTheme('dark');
        } else if (appearance === 'light' && theme !== 'light') {
          setTheme('light');
        }

        console.log(`🎨 Synced theme with system appearance: ${appearance}`);
      } catch (error) {
        // Silently fail on non-Tauri environments
        console.log('System appearance sync not available');
      }
    };

    syncSystemAppearance();
  }, []);

  // Monitor system appearance changes
  useSystemAppearance((appearance) => {
    if (appearance === 'dark' && theme !== 'dark') {
      setTheme('dark');
    } else if (appearance === 'light' && theme !== 'light') {
      setTheme('light');
    }
  });

  return null;
}
