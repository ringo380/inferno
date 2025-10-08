'use client';

import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

export type VibrancyEffect =
  | 'sidebar'
  | 'titlebar'
  | 'menu'
  | 'popover'
  | 'sheet'
  | 'hudWindow'
  | 'fullScreen'
  | 'tooltip'
  | 'contentBackground'
  | 'underWindowBackground'
  | 'underPageBackground';

/**
 * Hook to apply macOS window vibrancy effects
 * Only works on macOS - gracefully no-ops on other platforms
 */
export function useVibrancy(effect: VibrancyEffect = 'contentBackground') {
  useEffect(() => {
    const applyEffect = async () => {
      try {
        const window = getCurrentWindow();
        await invoke('apply_vibrancy', {
          window,
          effect,
        });
        console.log(`🎨 Applied vibrancy effect: ${effect}`);
      } catch (error) {
        // Silently fail on non-macOS platforms
        console.log('Vibrancy not available on this platform');
      }
    };

    applyEffect();
  }, [effect]);
}

/**
 * Hook to detect and respond to system appearance changes (light/dark mode)
 */
export function useSystemAppearance(onChange?: (appearance: 'light' | 'dark') => void) {
  useEffect(() => {
    const checkAppearance = async () => {
      try {
        const appearance = await invoke<string>('get_system_appearance');
        console.log(`🎨 System appearance: ${appearance}`);
        onChange?.(appearance as 'light' | 'dark');
      } catch (error) {
        console.log('System appearance detection not available');
      }
    };

    checkAppearance();

    // Listen for appearance change events
    const interval = setInterval(checkAppearance, 5000);

    return () => clearInterval(interval);
  }, [onChange]);
}
