'use client';

import { ReactNode } from 'react';
import { useNavigation } from '@/contexts/navigation-context';
import { NavigationBar } from '@/components/navigation/navigation-bar';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { cn } from '@/lib/utils';
import { LucideIcon } from 'lucide-react';

interface PageAction {
  label: string;
  onClick: () => void;
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  size?: 'default' | 'sm' | 'lg' | 'icon';
  icon?: LucideIcon;
  disabled?: boolean;
  loading?: boolean;
}

interface PageHeaderProps {
  title?: string;
  description?: string;
  icon?: LucideIcon;
  badge?: {
    text: string;
    variant?: 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning';
  };
  actions?: PageAction[];
  children?: ReactNode;
  className?: string;
  showNavigation?: boolean;
  lastUpdated?: Date | string;
  status?: {
    text: string;
    variant: 'success' | 'warning' | 'error' | 'info';
  };
}

export function PageHeader({
  title,
  description,
  icon: Icon,
  badge,
  actions = [],
  children,
  className,
  showNavigation = true,
  lastUpdated,
  status
}: PageHeaderProps) {
  const { currentPage } = useNavigation();

  // Use provided title/description or fall back to navigation context
  const displayTitle = title || currentPage?.title;
  const displayDescription = description || currentPage?.description;

  const statusColors = {
    success: 'text-green-600 bg-green-50 border-green-200 dark:text-green-400 dark:bg-green-950 dark:border-green-800',
    warning: 'text-yellow-600 bg-yellow-50 border-yellow-200 dark:text-yellow-400 dark:bg-yellow-950 dark:border-yellow-800',
    error: 'text-red-600 bg-red-50 border-red-200 dark:text-red-400 dark:bg-red-950 dark:border-red-800',
    info: 'text-blue-600 bg-blue-50 border-blue-200 dark:text-blue-400 dark:bg-blue-950 dark:border-blue-800'
  };

  return (
    <div className={cn('space-y-4', className)}>
      {/* Navigation Bar */}
      {showNavigation && (
        <NavigationBar
          variant="full"
          showTitle={false}
          showDescription={false}
          className="pb-2"
        />
      )}

      {/* Page Header Content */}
      <div className="space-y-4">
        {/* Title Section */}
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-3 mb-2">
              {Icon && (
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
                  <Icon className="h-5 w-5" />
                </div>
              )}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  {displayTitle && (
                    <h1 className="text-2xl font-bold tracking-tight text-foreground truncate">
                      {displayTitle}
                    </h1>
                  )}
                  {badge && (
                    <Badge variant={badge.variant || 'secondary'} className="text-xs">
                      {badge.text}
                    </Badge>
                  )}
                </div>

                {displayDescription && (
                  <p className="text-muted-foreground text-base">
                    {displayDescription}
                  </p>
                )}

                {/* Status and Last Updated */}
                <div className="flex items-center gap-4 mt-2">
                  {status && (
                    <div className={cn(
                      'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border',
                      statusColors[status.variant]
                    )}>
                      {status.text}
                    </div>
                  )}

                  {lastUpdated && (
                    <span className="text-xs text-muted-foreground">
                      Last updated: {
                        typeof lastUpdated === 'string'
                          ? lastUpdated
                          : lastUpdated.toLocaleString()
                      }
                    </span>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Actions */}
          {actions.length > 0 && (
            <div className="flex items-center gap-2 flex-shrink-0">
              {actions.map((action, index) => (
                <Button
                  key={index}
                  variant={action.variant || 'default'}
                  size={action.size || 'default'}
                  onClick={action.onClick}
                  disabled={action.disabled || action.loading}
                  className={cn(
                    action.icon && !action.label ? 'h-9 w-9 p-0' : '',
                    'whitespace-nowrap'
                  )}
                >
                  {action.loading ? (
                    <div className="animate-spin h-4 w-4 border-2 border-current border-t-transparent rounded-full" />
                  ) : action.icon ? (
                    <action.icon className={cn('h-4 w-4', action.label ? 'mr-2' : '')} />
                  ) : null}
                  {action.label}
                </Button>
              ))}
            </div>
          )}
        </div>

        {/* Custom Children Content */}
        {children && (
          <>
            <Separator />
            <div>{children}</div>
          </>
        )}
      </div>
    </div>
  );
}

// Specialized variants for common use cases

interface DashboardPageHeaderProps extends Omit<PageHeaderProps, 'showNavigation'> {
  showNavigation?: boolean;
}

export function DashboardPageHeader(props: DashboardPageHeaderProps) {
  return <PageHeader {...props} showNavigation={props.showNavigation ?? true} />;
}

interface ModalPageHeaderProps extends Omit<PageHeaderProps, 'showNavigation'> {}

export function ModalPageHeader(props: ModalPageHeaderProps) {
  return <PageHeader {...props} showNavigation={false} />;
}

interface SimplePageHeaderProps {
  title: string;
  description?: string;
  children?: ReactNode;
  className?: string;
}

export function SimplePageHeader({ title, description, children, className }: SimplePageHeaderProps) {
  return (
    <div className={cn('space-y-4', className)}>
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          {title}
        </h1>
        {description && (
          <p className="text-muted-foreground text-base mt-1">
            {description}
          </p>
        )}
      </div>
      {children && (
        <>
          <Separator />
          <div>{children}</div>
        </>
      )}
    </div>
  );
}