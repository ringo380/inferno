'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { LucideIcon, TrendingUp, TrendingDown, Minus } from 'lucide-react';
import { cn } from '@/lib/utils';

interface MetricCardProps {
  title: string;
  value: string;
  description: string;
  icon: LucideIcon;
  trend: 'up' | 'down' | 'stable';
  color: 'blue' | 'green' | 'yellow' | 'red';
}

const colorVariants = {
  blue: 'text-blue-600 dark:text-blue-400',
  green: 'text-green-600 dark:text-green-400',
  yellow: 'text-yellow-600 dark:text-yellow-400',
  red: 'text-red-600 dark:text-red-400',
};

const trendVariants = {
  up: { icon: TrendingUp, color: 'text-green-600 dark:text-green-400' },
  down: { icon: TrendingDown, color: 'text-red-600 dark:text-red-400' },
  stable: { icon: Minus, color: 'text-gray-600 dark:text-gray-400' },
};

export function MetricCard({
  title,
  value,
  description,
  icon: Icon,
  trend,
  color,
}: MetricCardProps) {
  const TrendIcon = trendVariants[trend].icon;
  const trendColor = trendVariants[trend].color;

  return (
    <Card className="card-hover">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className={cn('h-4 w-4', colorVariants[color])} />
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">{value}</div>
        <div className="flex items-center space-x-1 text-xs text-muted-foreground mt-1">
          <TrendIcon className={cn('h-3 w-3', trendColor)} />
          <span>{description}</span>
        </div>
      </CardContent>
    </Card>
  );
}