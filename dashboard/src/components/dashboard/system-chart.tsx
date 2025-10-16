'use client';

import { useState, useEffect, useRef, useMemo, memo } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useTheme } from 'next-themes';
import { useRealTimeSystemMetrics } from '@/hooks/use-real-time-events';
import { useSystemInfo, useInfernoMetrics } from '@/hooks/use-tauri-api';

interface DataPoint {
  time: string;
  cpu: number;
  memory: number;
  inference: number;
}

const MAX_DATA_POINTS = 30; // Keep last 30 data points

export const SystemChart = memo(function SystemChart() {
  const { theme } = useTheme();
  const isDark = theme === 'dark';

  // Get real-time metrics
  const { metrics: realTimeMetrics } = useRealTimeSystemMetrics();
  const { data: systemInfo } = useSystemInfo();
  const { data: infernoMetrics } = useInfernoMetrics();

  // Historical data points
  const [dataPoints, setDataPoints] = useState<DataPoint[]>([]);
  const lastInferenceCount = useRef<number>(0);

  // Update data points when new metrics arrive
  useEffect(() => {
    if (realTimeMetrics && systemInfo) {
      const now = new Date();
      const timeLabel = now.toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit',
        hour12: false
      });

      // Calculate memory percentage
      const memoryPercent = systemInfo.total_memory > 0
        ? (realTimeMetrics.memory_usage / systemInfo.total_memory) * 100
        : 0;

      // Calculate inference rate (inferences per minute)
      const currentInferenceCount = infernoMetrics?.inference_count ?? 0;
      let inferenceRate = 0;

      if (dataPoints.length > 0 && currentInferenceCount > lastInferenceCount.current) {
        // Calculate inferences since last data point (roughly 3 seconds ago)
        // Extrapolate to per-minute rate
        const inferencesSinceLastPoint = currentInferenceCount - lastInferenceCount.current;
        inferenceRate = (inferencesSinceLastPoint / 3) * 60; // Convert 3-second rate to per-minute
      }

      lastInferenceCount.current = currentInferenceCount;

      const newPoint: DataPoint = {
        time: timeLabel,
        cpu: Math.round(realTimeMetrics.cpu_usage * 10) / 10,
        memory: Math.round(memoryPercent * 10) / 10,
        inference: Math.round(inferenceRate),
      };

      setDataPoints(prev => {
        const updated = [...prev, newPoint];
        // Keep only last MAX_DATA_POINTS
        return updated.slice(-MAX_DATA_POINTS);
      });
    }
  }, [realTimeMetrics, systemInfo, infernoMetrics, dataPoints.length]);

  // Use mock data if no real-time data available yet
  const displayData = useMemo(
    () => dataPoints.length > 0 ? dataPoints : [{ time: 'Waiting...', cpu: 0, memory: 0, inference: 0 }],
    [dataPoints]
  );

  return (
    <div className="h-[300px] w-full transition-opacity duration-300 ease-in-out">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart
          data={displayData}
          margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
        >
          <CartesianGrid
            strokeDasharray="3 3"
            stroke={isDark ? '#374151' : '#e5e7eb'}
          />
          <XAxis
            dataKey="time"
            stroke={isDark ? '#9ca3af' : '#6b7280'}
            fontSize={12}
          />
          <YAxis
            stroke={isDark ? '#9ca3af' : '#6b7280'}
            fontSize={12}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: isDark ? '#1f2937' : '#ffffff',
              border: `1px solid ${isDark ? '#374151' : '#e5e7eb'}`,
              borderRadius: '8px',
              color: isDark ? '#f9fafb' : '#111827',
            }}
            labelStyle={{ color: isDark ? '#f9fafb' : '#111827' }}
            animationDuration={150}
          />
          <Legend />
          <Line
            type="monotone"
            dataKey="cpu"
            stroke="#3b82f6"
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 4 }}
            name="CPU %"
            isAnimationActive={false}
          />
          <Line
            type="monotone"
            dataKey="memory"
            stroke="#10b981"
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 4 }}
            name="Memory %"
            isAnimationActive={false}
          />
          <Line
            type="monotone"
            dataKey="inference"
            stroke="#f59e0b"
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 4 }}
            name="Inferences/hr"
            isAnimationActive={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
});