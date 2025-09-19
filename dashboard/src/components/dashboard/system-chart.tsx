'use client';

import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useTheme } from 'next-themes';

// Mock data for the chart
const data = [
  { time: '00:00', cpu: 35, memory: 45, inference: 120 },
  { time: '04:00', cpu: 28, memory: 42, inference: 98 },
  { time: '08:00', cpu: 45, memory: 55, inference: 245 },
  { time: '12:00', cpu: 72, memory: 68, inference: 387 },
  { time: '16:00', cpu: 65, memory: 62, inference: 342 },
  { time: '20:00', cpu: 52, memory: 58, inference: 298 },
  { time: '24:00', cpu: 41, memory: 48, inference: 156 },
];

export function SystemChart() {
  const { theme } = useTheme();
  const isDark = theme === 'dark';

  return (
    <div className="h-[300px] w-full">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
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
          />
          <Legend />
          <Line
            type="monotone"
            dataKey="cpu"
            stroke="#3b82f6"
            strokeWidth={2}
            dot={{ fill: '#3b82f6', strokeWidth: 2 }}
            name="CPU %"
          />
          <Line
            type="monotone"
            dataKey="memory"
            stroke="#10b981"
            strokeWidth={2}
            dot={{ fill: '#10b981', strokeWidth: 2 }}
            name="Memory %"
          />
          <Line
            type="monotone"
            dataKey="inference"
            stroke="#f59e0b"
            strokeWidth={2}
            dot={{ fill: '#f59e0b', strokeWidth: 2 }}
            name="Inferences/hr"
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}