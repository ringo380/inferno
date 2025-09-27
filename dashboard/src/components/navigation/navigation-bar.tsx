'use client';

import { ArrowLeft, ArrowRight, Home, MoreHorizontal } from 'lucide-react';
import { useNavigation } from '@/contexts/navigation-context';
import { Button } from '@/components/ui/button';
import { Breadcrumbs, CompactBreadcrumbs } from './breadcrumbs';
import { cn } from '@/lib/utils';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Badge } from '@/components/ui/badge';

interface NavigationBarProps {
  className?: string;
  variant?: 'full' | 'compact' | 'minimal';
  showTitle?: boolean;
  showDescription?: boolean;
}

export function NavigationBar({
  className,
  variant = 'full',
  showTitle = true,
  showDescription = true
}: NavigationBarProps) {
  const {
    currentPage,
    canGoBack,
    canGoForward,
    goBack,
    goForward,
    goHome,
    recentPages,
    navigateTo
  } = useNavigation();

  const handleBackClick = () => {
    goBack();
  };

  const handleForwardClick = () => {
    goForward();
  };

  const handleHomeClick = () => {
    goHome();
  };

  if (variant === 'minimal') {
    return (
      <div className={cn('flex items-center space-x-2', className)}>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleBackClick}
          disabled={!canGoBack}
          className="h-8 w-8 p-0"
          title="Go back (Alt+Left)"
        >
          <ArrowLeft className="h-4 w-4" />
        </Button>

        <Button
          variant="ghost"
          size="sm"
          onClick={handleForwardClick}
          disabled={!canGoForward}
          className="h-8 w-8 p-0"
          title="Go forward (Alt+Right)"
        >
          <ArrowRight className="h-4 w-4" />
        </Button>

        <Button
          variant="ghost"
          size="sm"
          onClick={handleHomeClick}
          className="h-8 w-8 p-0"
          title="Go home (Cmd+H)"
        >
          <Home className="h-4 w-4" />
        </Button>
      </div>
    );
  }

  if (variant === 'compact') {
    return (
      <div className={cn('flex items-center justify-between py-2', className)}>
        <div className="flex items-center space-x-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleBackClick}
            disabled={!canGoBack}
            className="h-8 w-8 p-0"
            title="Go back (Alt+Left)"
          >
            <ArrowLeft className="h-4 w-4" />
          </Button>

          <Button
            variant="ghost"
            size="sm"
            onClick={handleHomeClick}
            className="h-8 w-8 p-0"
            title="Go home (Cmd+H)"
          >
            <Home className="h-4 w-4" />
          </Button>

          <CompactBreadcrumbs />
        </div>

        {recentPages.length > 0 && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-56">
              <div className="px-2 py-1.5 text-xs font-semibold text-muted-foreground">
                Recent Pages
              </div>
              <DropdownMenuSeparator />
              {recentPages.slice(0, 5).map((page) => (
                <DropdownMenuItem
                  key={page.path}
                  onClick={() => navigateTo(page.path)}
                  className="flex items-center justify-between"
                >
                  <span className="truncate">{page.title}</span>
                  <Badge variant="secondary" className="ml-2 text-xs">
                    {new Date(page.timestamp).toLocaleTimeString([], {
                      hour: '2-digit',
                      minute: '2-digit'
                    })}
                  </Badge>
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>
    );
  }

  return (
    <div className={cn('space-y-3', className)}>
      {/* Navigation Controls */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleBackClick}
            disabled={!canGoBack}
            className="h-9 px-3"
            title="Go back (Alt+Left)"
          >
            <ArrowLeft className="h-4 w-4 mr-1" />
            <span className="hidden sm:inline">Back</span>
          </Button>

          <Button
            variant="ghost"
            size="sm"
            onClick={handleForwardClick}
            disabled={!canGoForward}
            className="h-9 px-3"
            title="Go forward (Alt+Right)"
          >
            <ArrowRight className="h-4 w-4 mr-1" />
            <span className="hidden sm:inline">Forward</span>
          </Button>

          <Button
            variant="ghost"
            size="sm"
            onClick={handleHomeClick}
            className="h-9 px-3"
            title="Go home (Cmd+H)"
          >
            <Home className="h-4 w-4 mr-1" />
            <span className="hidden sm:inline">Home</span>
          </Button>
        </div>

        {recentPages.length > 0 && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="h-9">
                <MoreHorizontal className="h-4 w-4 mr-1" />
                Recent
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-64">
              <div className="px-2 py-1.5 text-xs font-semibold text-muted-foreground">
                Recently Visited Pages
              </div>
              <DropdownMenuSeparator />
              {recentPages.slice(0, 8).map((page) => (
                <DropdownMenuItem
                  key={page.path}
                  onClick={() => navigateTo(page.path)}
                  className="flex flex-col items-start space-y-1 py-2"
                >
                  <div className="flex items-center justify-between w-full">
                    <span className="font-medium truncate">{page.title}</span>
                    <Badge variant="secondary" className="text-xs">
                      {new Date(page.timestamp).toLocaleTimeString([], {
                        hour: '2-digit',
                        minute: '2-digit'
                      })}
                    </Badge>
                  </div>
                  {page.description && (
                    <p className="text-xs text-muted-foreground truncate w-full">
                      {page.description}
                    </p>
                  )}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>

      {/* Breadcrumbs */}
      <Breadcrumbs />

      {/* Page Title and Description */}
      {(showTitle || showDescription) && currentPage && (
        <div className="space-y-1">
          {showTitle && (
            <h1 className="text-2xl font-bold tracking-tight text-foreground">
              {currentPage.title}
            </h1>
          )}
          {showDescription && currentPage.description && (
            <p className="text-muted-foreground">
              {currentPage.description}
            </p>
          )}
        </div>
      )}
    </div>
  );
}

// Keyboard shortcut hook for navigation
export function useNavigationShortcuts() {
  const { goBack, goForward, goHome, canGoBack, canGoForward } = useNavigation();

  // This will be integrated with the existing keyboard shortcuts provider
  return {
    goBack: canGoBack ? goBack : undefined,
    goForward: canGoForward ? goForward : undefined,
    goHome,
  };
}