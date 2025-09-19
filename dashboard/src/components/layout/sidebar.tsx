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

const navigationItems = [
  {
    title: 'Overview',
    href: '/',
    icon: Home,
    badge: null,
  },
  {
    title: 'Models',
    href: '/models',
    icon: Brain,
    badge: '12',
  },
  {
    title: 'Inference',
    href: '/inference',
    icon: Play,
    badge: null,
  },
  {
    title: 'Monitoring',
    href: '/monitoring',
    icon: BarChart3,
    badge: null,
  },
  {
    title: 'Batch Jobs',
    href: '/batch-jobs',
    icon: Clock,
    badge: '3',
  },
  {
    title: 'Performance',
    href: '/performance',
    icon: Zap,
    badge: null,
  },
  {
    title: 'Security',
    href: '/security',
    icon: Shield,
    badge: '!',
    badgeVariant: 'warning' as const,
  },
  {
    title: 'Multi-Tenancy',
    href: '/tenants',
    icon: Users,
    badge: '5',
  },
  {
    title: 'Data Pipeline',
    href: '/pipeline',
    icon: Database,
    badge: null,
  },
  {
    title: 'Observability',
    href: '/observability',
    icon: Activity,
    badge: null,
  },
  {
    title: 'Versioning',
    href: '/versioning',
    icon: GitBranch,
    badge: null,
  },
  {
    title: 'Settings',
    href: '/settings',
    icon: Settings,
    badge: null,
  },
];

interface SidebarProps {
  className?: string;
}

export function Sidebar({ className }: SidebarProps) {
  const [collapsed, setCollapsed] = useState(false);
  const pathname = usePathname();

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
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">CPU Usage</span>
                <span className="font-medium">45%</span>
              </div>
              <div className="w-full bg-secondary rounded-full h-2">
                <div className="bg-green-500 h-2 rounded-full w-[45%]"></div>
              </div>

              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Memory</span>
                <span className="font-medium">2.1GB</span>
              </div>
              <div className="w-full bg-secondary rounded-full h-2">
                <div className="bg-blue-500 h-2 rounded-full w-[60%]"></div>
              </div>

              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Active Models</span>
                <span className="font-medium text-green-600">3</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}