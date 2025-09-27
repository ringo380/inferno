'use client';

import { ChevronRight, Home } from 'lucide-react';
import Link from 'next/link';
import { useNavigation, BreadcrumbItem } from '@/contexts/navigation-context';
import { cn } from '@/lib/utils';

interface BreadcrumbsProps {
  className?: string;
  showHomeIcon?: boolean;
  maxItems?: number;
}

export function Breadcrumbs({ className, showHomeIcon = true, maxItems = 5 }: BreadcrumbsProps) {
  const { breadcrumbs } = useNavigation();

  if (!breadcrumbs || breadcrumbs.length === 0) {
    return null;
  }

  // Truncate breadcrumbs if they exceed maxItems
  const displayBreadcrumbs = breadcrumbs.length > maxItems
    ? [
        breadcrumbs[0], // Always show first (home)
        { label: '...', href: '', isEllipsis: true } as BreadcrumbItem & { isEllipsis: boolean },
        ...breadcrumbs.slice(-(maxItems - 2)) // Show last few items
      ]
    : breadcrumbs;

  return (
    <nav
      className={cn('flex items-center space-x-1 text-sm text-muted-foreground', className)}
      aria-label="Breadcrumb navigation"
    >
      <ol className="flex items-center space-x-1">
        {displayBreadcrumbs.map((item, index) => {
          const isLast = index === displayBreadcrumbs.length - 1;
          const isEllipsis = 'isEllipsis' in item && item.isEllipsis;
          const isHome = index === 0 && item.href === '/';

          return (
            <li key={`${item.href}-${index}`} className="flex items-center">
              {index > 0 && (
                <ChevronRight className="mx-1 h-3 w-3 flex-shrink-0 text-muted-foreground/60" />
              )}

              {isEllipsis ? (
                <span className="px-1 text-muted-foreground/60">...</span>
              ) : isLast ? (
                <span
                  className="font-medium text-foreground truncate max-w-[200px]"
                  aria-current="page"
                >
                  {isHome && showHomeIcon ? (
                    <div className="flex items-center space-x-1">
                      <Home className="h-3 w-3" />
                      <span className="hidden sm:inline">{item.label}</span>
                    </div>
                  ) : (
                    item.label
                  )}
                </span>
              ) : (
                <Link
                  href={item.href}
                  className={cn(
                    'hover:text-foreground transition-colors truncate max-w-[150px]',
                    'focus:outline-none focus:ring-2 focus:ring-primary/20 focus:text-foreground rounded px-1',
                    isHome && showHomeIcon ? 'flex items-center space-x-1' : ''
                  )}
                  title={item.label}
                >
                  {isHome && showHomeIcon ? (
                    <>
                      <Home className="h-3 w-3" />
                      <span className="hidden sm:inline">{item.label}</span>
                    </>
                  ) : (
                    item.label
                  )}
                </Link>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}

// Alternative compact version for mobile
export function CompactBreadcrumbs({ className }: { className?: string }) {
  const { breadcrumbs, goBack, canGoBack } = useNavigation();

  if (!breadcrumbs || breadcrumbs.length === 0) {
    return null;
  }

  const currentPage = breadcrumbs[breadcrumbs.length - 1];
  const parentPage = breadcrumbs.length > 1 ? breadcrumbs[breadcrumbs.length - 2] : null;

  return (
    <nav className={cn('flex items-center space-x-2 text-sm', className)}>
      {parentPage && canGoBack && (
        <button
          onClick={goBack}
          className="flex items-center space-x-1 text-muted-foreground hover:text-foreground transition-colors"
        >
          <ChevronRight className="h-3 w-3 rotate-180" />
          <span className="truncate max-w-[100px]">{parentPage.label}</span>
        </button>
      )}

      {parentPage && (
        <ChevronRight className="h-3 w-3 text-muted-foreground/60" />
      )}

      <span className="font-medium text-foreground truncate max-w-[150px]">
        {currentPage?.label}
      </span>
    </nav>
  );
}

// Structured data breadcrumbs for SEO
export function StructuredBreadcrumbs() {
  const { breadcrumbs } = useNavigation();

  if (!breadcrumbs || breadcrumbs.length === 0) {
    return null;
  }

  const structuredData = {
    '@context': 'https://schema.org',
    '@type': 'BreadcrumbList',
    itemListElement: breadcrumbs.map((item, index) => ({
      '@type': 'ListItem',
      position: index + 1,
      name: item.label,
      item: `${typeof window !== 'undefined' ? window.location.origin : ''}${item.href}`
    }))
  };

  return (
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(structuredData) }}
    />
  );
}