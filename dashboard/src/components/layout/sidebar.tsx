'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { cn } from '@/lib/utils';
import {
  Home,
  Brain,
  Play,
  BarChart3,
  Clock,
  Settings,
  Shield,
  Users,
  Database,
  Zap,
  GitBranch,
  Activity,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useModels, useLoadedModels, useSystemInfo, useActiveProcesses } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';

const baseNavigationItems = [
  {
    title: 'Overview',
    href: '/',
    icon: Home,
    badgeKey: null,
  },
  {
    title: 'Models',
    href: '/models',
    icon: Brain,
    badgeKey: 'models',
  },
  {
    title: 'Inference',
    href: '/inference',
    icon: Play,
    badgeKey: null,
  },
  {
    title: 'Monitoring',
    href: '/monitoring',
    icon: BarChart3,
    badgeKey: null,
  },
  {
    title: 'Batch Jobs',
    href: '/batch',
    icon: Clock,
    badgeKey: 'batchJobs',
  },
  {
    title: 'Performance',
    href: '/performance',
    icon: Zap,
    badgeKey: null,
  },
  {
    title: 'Security',
    href: '/security',
    icon: Shield,
    badgeKey: 'security',
    badgeVariant: 'warning' as const,
  },
  {
    title: 'Multi-Tenancy',
    href: '/tenants',
    icon: Users,
    badgeKey: 'tenants',
  },
  {
    title: 'Data Pipeline',
    href: '/pipeline',
    icon: Database,
    badgeKey: null,
  },
  {
    title: 'Observability',
    href: '/observability',
    icon: Activity,
    badgeKey: null,
  },
  {
    title: 'Versioning',
    href: '/versioning',
    icon: GitBranch,
    badgeKey: null,
  },
  {
    title: 'Settings',
    href: '/settings',
    icon: Settings,
    badgeKey: null,
  },
];

interface SidebarProps {
  className?: string;
}

export function Sidebar({ className }: SidebarProps) {
  const [collapsed, setCollapsed] = useState(false);
  const pathname = usePathname();

  // Fetch real data
  const { data: models, isLoading: modelsLoading } = useModels();
  const { data: loadedModels, isLoading: loadedModelsLoading } = useLoadedModels();
  const { data: systemInfo, isLoading: systemLoading } = useSystemInfo();
  const { data: activeProcesses, isLoading: activeProcessesLoading } = useActiveProcesses();

  // Calculate badge values based on real data
  const badgeValues = {
    models: models?.length ?? 0,
    batchJobs: activeProcesses?.batch_jobs ?? 0,
    security: 0, // Will be implemented later
    tenants: 1, // Default single tenant for now
  };

  // Create navigation items with real badge values
  const navigationItems = baseNavigationItems.map(item => ({
    ...item,
    badge: item.badgeKey && badgeValues[item.badgeKey as keyof typeof badgeValues]
      ? String(badgeValues[item.badgeKey as keyof typeof badgeValues])
      : item.badgeKey === 'security' && badgeValues.security === 0
        ? null
        : item.badgeKey === 'security'
          ? '!'
          : null,
  }));

  return (
    <div className={cn(
      'relative flex flex-col border-r bg-card transition-all duration-300',
      collapsed ? 'w-16' : 'w-64',
      className
    )}>
      {/* Collapse Toggle */}
      <Button
        variant="ghost"
        size="icon"
        className="absolute -right-4 top-6 z-10 h-8 w-8 rounded-full border bg-background shadow-md hover:bg-accent"
        onClick={() => setCollapsed(!collapsed)}
      >
        {collapsed ? (
          <ChevronRight className="h-4 w-4" />
        ) : (
          <ChevronLeft className="h-4 w-4" />
        )}
      </Button>

      {/* Navigation */}
      <nav className="flex-1 p-4 space-y-2">
        {navigationItems.map((item) => {
          const isActive = pathname === item.href;
          const Icon = item.icon;

          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                'flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-all hover:bg-accent',
                isActive
                  ? 'bg-accent text-accent-foreground font-medium'
                  : 'text-muted-foreground hover:text-foreground',
                collapsed && 'justify-center px-2'
              )}
            >
              <Icon className="h-4 w-4 flex-shrink-0" />
              {!collapsed && (
                <>
                  <span className="flex-1">{item.title}</span>
                  {item.badge && (
                    <Badge
                      variant={item.badgeVariant || 'secondary'}
                      className="h-5 text-xs"
                    >
                      {item.badge}
                    </Badge>
                  )}
                </>
              )}
            </Link>
          );
        })}
      </nav>

      {/* Status Panel */}
      {!collapsed && (
        <div className="p-4 border-t">
          <div className="space-y-3">
            <div className="text-xs font-medium text-muted-foreground">
              System Status
            </div>
            {systemLoading ? (
              <div className="space-y-3">
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Skeleton className="h-3 w-16" />
                    <Skeleton className="h-3 w-8" />
                  </div>
                  <Skeleton className="h-2 w-full rounded-full" />
                </div>
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Skeleton className="h-3 w-12" />
                    <Skeleton className="h-3 w-12" />
                  </div>
                  <Skeleton className="h-2 w-full rounded-full" />
                </div>
                <div className="flex items-center justify-between">
                  <Skeleton className="h-3 w-20" />
                  <Skeleton className="h-3 w-4" />
                </div>
              </div>
            ) : (
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">CPU Usage</span>
                  <span className="font-medium">
                    {systemInfo?.cpu_usage ? `${systemInfo.cpu_usage.toFixed(1)}%` : '0%'}
                  </span>
                </div>
                <div className="w-full bg-secondary rounded-full h-2">
                  <div
                    className="bg-green-500 h-2 rounded-full transition-all duration-500"
                    style={{ width: `${systemInfo?.cpu_usage || 0}%` }}
                  ></div>
                </div>

                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Memory</span>
                  <span className="font-medium">
                    {systemInfo ?
                      `${(systemInfo.used_memory / (1024 * 1024 * 1024)).toFixed(1)}GB` :
                      '0GB'
                    }
                  </span>
                </div>
                <div className="w-full bg-secondary rounded-full h-2">
                  <div
                    className="bg-blue-500 h-2 rounded-full transition-all duration-500"
                    style={{
                      width: `${systemInfo ? (systemInfo.used_memory / systemInfo.total_memory) * 100 : 0}%`
                    }}
                  ></div>
                </div>

                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Active Models</span>
                  <span className="font-medium text-green-600">
                    {loadedModelsLoading ? (
                      <Skeleton className="h-3 w-4 inline-block" />
                    ) : (
                      loadedModels?.length || 0
                    )}
                  </span>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}