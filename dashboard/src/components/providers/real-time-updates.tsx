'use client';

import React, { createContext, useContext, useState, useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { toast } from 'react-hot-toast';

interface RealTimeUpdateSettings {
  enabled: boolean;
  mode: 'normal' | 'live' | 'minimal';
  intervals: {
    critical: number;    // Loaded models, active processes
    normal: number;      // System info, metrics
    background: number;  // Activities, notifications
  };
}

interface RealTimeUpdateContext {
  settings: RealTimeUpdateSettings;
  updateSettings: (newSettings: Partial<RealTimeUpdateSettings>) => void;
  toggleRealTime: () => void;
  forceRefresh: () => void;
  lastUpdateTime: Date | null;
  isUpdating: boolean;
}

const defaultSettings: RealTimeUpdateSettings = {
  enabled: true,
  mode: 'normal',
  intervals: {
    critical: 2000,    // 2 seconds
    normal: 5000,      // 5 seconds
    background: 10000, // 10 seconds
  },
};

const RealTimeContext = createContext<RealTimeUpdateContext | undefined>(undefined);

export function RealTimeUpdateProvider({ children }: { children: React.ReactNode }) {
  const [settings, setSettings] = useState<RealTimeUpdateSettings>(defaultSettings);
  const [lastUpdateTime, setLastUpdateTime] = useState<Date | null>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const queryClient = useQueryClient();

  // Update intervals based on mode
  useEffect(() => {
    const intervals = {
      live: { critical: 1000, normal: 2000, background: 5000 },
      normal: { critical: 2000, normal: 5000, background: 10000 },
      minimal: { critical: 5000, normal: 15000, background: 30000 },
    };

    setSettings(prev => ({
      ...prev,
      intervals: intervals[prev.mode],
    }));
  }, [settings.mode]);

  // Auto-refresh logic
  useEffect(() => {
    if (!settings.enabled) return;

    const interval = setInterval(async () => {
      setIsUpdating(true);
      try {
        // Invalidate critical data queries
        await queryClient.invalidateQueries({
          queryKey: ['loaded-models'],
          refetchType: 'active'
        });
        await queryClient.invalidateQueries({
          queryKey: ['active-processes'],
          refetchType: 'active'
        });

        setLastUpdateTime(new Date());
      } catch (error) {
        console.error('Real-time update failed:', error);
      } finally {
        setIsUpdating(false);
      }
    }, settings.intervals.critical);

    return () => clearInterval(interval);
  }, [settings.enabled, settings.intervals.critical, queryClient]);

  const updateSettings = (newSettings: Partial<RealTimeUpdateSettings>) => {
    setSettings(prev => ({ ...prev, ...newSettings }));
  };

  const toggleRealTime = () => {
    const newEnabled = !settings.enabled;
    setSettings(prev => ({ ...prev, enabled: newEnabled }));

    if (newEnabled) {
      toast.success('Real-time updates enabled');
    } else {
      toast.success('Real-time updates disabled');
    }
  };

  const forceRefresh = async () => {
    setIsUpdating(true);
    try {
      await queryClient.invalidateQueries();
      setLastUpdateTime(new Date());
      toast.success('All data refreshed');
    } catch (error) {
      toast.error('Failed to refresh data');
    } finally {
      setIsUpdating(false);
    }
  };

  const value: RealTimeUpdateContext = {
    settings,
    updateSettings,
    toggleRealTime,
    forceRefresh,
    lastUpdateTime,
    isUpdating,
  };

  return (
    <RealTimeContext.Provider value={value}>
      {children}
    </RealTimeContext.Provider>
  );
}

export function useRealTimeUpdates() {
  const context = useContext(RealTimeContext);
  if (context === undefined) {
    throw new Error('useRealTimeUpdates must be used within a RealTimeUpdateProvider');
  }
  return context;
}