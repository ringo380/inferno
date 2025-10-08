'use client';

import { useVibrancy } from '@/hooks/use-vibrancy';

/**
 * Client component wrapper that applies macOS window vibrancy effects
 */
export function VibrancyWrapper() {
  useVibrancy('contentBackground');
  return null;
}
