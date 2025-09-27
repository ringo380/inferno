'use client';

import { createContext, useContext, useEffect, useState, ReactNode, useCallback } from 'react';
import { useRouter, usePathname } from 'next/navigation';

export interface NavigationItem {
  path: string;
  title: string;
  description?: string;
  timestamp: number;
  breadcrumbs: BreadcrumbItem[];
}

export interface BreadcrumbItem {
  label: string;
  href: string;
  isActive?: boolean;
}

interface NavigationContextType {
  // Current page info
  currentPage: NavigationItem | null;
  setCurrentPage: (page: Partial<NavigationItem>) => void;

  // Navigation history
  history: NavigationItem[];
  canGoBack: boolean;
  canGoForward: boolean;

  // Navigation actions
  goBack: () => void;
  goForward: () => void;
  goHome: () => void;
  navigateTo: (path: string, options?: { title?: string; description?: string }) => void;

  // Recent pages
  recentPages: NavigationItem[];

  // Breadcrumbs
  breadcrumbs: BreadcrumbItem[];
}

const NavigationContext = createContext<NavigationContextType | undefined>(undefined);

// Page metadata configuration
const PAGE_CONFIG: Record<string, { title: string; description?: string; breadcrumbs: BreadcrumbItem[] }> = {
  '/': {
    title: 'Dashboard Overview',
    description: 'Enterprise AI/ML model management platform',
    breadcrumbs: [{ label: 'Home', href: '/', isActive: true }]
  },
  '/models': {
    title: 'Model Management',
    description: 'Manage and monitor your AI/ML models',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Models', href: '/models', isActive: true }
    ]
  },
  '/inference': {
    title: 'Inference Testing',
    description: 'Test real-time streaming inference with your models',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Inference', href: '/inference', isActive: true }
    ]
  },
  '/monitoring': {
    title: 'System Monitoring',
    description: 'Monitor system performance and resource usage',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Monitoring', href: '/monitoring', isActive: true }
    ]
  },
  '/batch': {
    title: 'Batch Jobs',
    description: 'Manage batch processing jobs and schedules',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Batch Jobs', href: '/batch', isActive: true }
    ]
  },
  '/performance': {
    title: 'Performance Analytics',
    description: 'Analyze model performance and optimization metrics',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Performance', href: '/performance', isActive: true }
    ]
  },
  '/security': {
    title: 'Security Management',
    description: 'Manage security policies and access controls',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Security', href: '/security', isActive: true }
    ]
  },
  '/tenants': {
    title: 'Multi-Tenancy',
    description: 'Manage tenant isolation and resource allocation',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Multi-Tenancy', href: '/tenants', isActive: true }
    ]
  },
  '/pipeline': {
    title: 'Data Pipeline',
    description: 'Configure and monitor data processing pipelines',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Data Pipeline', href: '/pipeline', isActive: true }
    ]
  },
  '/observability': {
    title: 'Observability',
    description: 'Monitor logs, metrics, and distributed traces',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Observability', href: '/observability', isActive: true }
    ]
  },
  '/versioning': {
    title: 'Model Versioning',
    description: 'Manage model versions and deployment history',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Versioning', href: '/versioning', isActive: true }
    ]
  },
  '/settings': {
    title: 'Settings',
    description: 'Configure application settings and preferences',
    breadcrumbs: [
      { label: 'Home', href: '/' },
      { label: 'Settings', href: '/settings', isActive: true }
    ]
  }
};

interface NavigationProviderProps {
  children: ReactNode;
}

const STORAGE_KEY = 'inferno-navigation-history';
const MAX_HISTORY_SIZE = 10;
const MAX_RECENT_SIZE = 5;

export function NavigationProvider({ children }: NavigationProviderProps) {
  const router = useRouter();
  const pathname = usePathname();

  const [currentPage, setCurrentPageState] = useState<NavigationItem | null>(null);
  const [history, setHistory] = useState<NavigationItem[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [recentPages, setRecentPages] = useState<NavigationItem[]>([]);

  // Load saved navigation state on mount
  useEffect(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const { history: savedHistory, recentPages: savedRecent } = JSON.parse(saved);
        setHistory(savedHistory || []);
        setRecentPages(savedRecent || []);
      }
    } catch (error) {
      console.warn('Failed to load navigation history:', error);
    }
  }, []);

  // Save navigation state to localStorage
  const saveNavigationState = useCallback((newHistory: NavigationItem[], newRecent: NavigationItem[]) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({
        history: newHistory,
        recentPages: newRecent
      }));
    } catch (error) {
      console.warn('Failed to save navigation history:', error);
    }
  }, []);

  // Set current page with automatic breadcrumb generation
  const setCurrentPage = useCallback((pageData: Partial<NavigationItem>) => {
    const config = PAGE_CONFIG[pathname];
    const page: NavigationItem = {
      path: pathname,
      title: pageData.title || config?.title || 'Unknown Page',
      description: pageData.description || config?.description,
      timestamp: Date.now(),
      breadcrumbs: pageData.breadcrumbs || config?.breadcrumbs || [
        { label: 'Home', href: '/' },
        { label: pageData.title || 'Page', href: pathname, isActive: true }
      ]
    };

    setCurrentPageState(page);

    // Add to history if it's a new page
    setHistory(prev => {
      const filtered = prev.filter(item => item.path !== pathname);
      const newHistory = [...filtered, page].slice(-MAX_HISTORY_SIZE);

      // Update recent pages (excluding current)
      const newRecent = [page, ...recentPages.filter(item => item.path !== pathname)]
        .slice(0, MAX_RECENT_SIZE);
      setRecentPages(newRecent);

      // Update history index to point to current page
      setHistoryIndex(newHistory.length - 1);

      // Save to localStorage
      saveNavigationState(newHistory, newRecent);

      return newHistory;
    });
  }, [pathname, recentPages, saveNavigationState]);

  // Auto-set page info when pathname changes
  useEffect(() => {
    const config = PAGE_CONFIG[pathname];
    if (config) {
      setCurrentPage({
        title: config.title,
        description: config.description,
        breadcrumbs: config.breadcrumbs
      });
    }
  }, [pathname, setCurrentPage]);

  // Navigation actions
  const canGoBack = historyIndex > 0;
  const canGoForward = historyIndex < history.length - 1;

  const goBack = useCallback(() => {
    if (canGoBack) {
      const newIndex = historyIndex - 1;
      const targetPage = history[newIndex];
      setHistoryIndex(newIndex);
      router.push(targetPage.path);
    }
  }, [canGoBack, historyIndex, history, router]);

  const goForward = useCallback(() => {
    if (canGoForward) {
      const newIndex = historyIndex + 1;
      const targetPage = history[newIndex];
      setHistoryIndex(newIndex);
      router.push(targetPage.path);
    }
  }, [canGoForward, historyIndex, history, router]);

  const goHome = useCallback(() => {
    router.push('/');
  }, [router]);

  const navigateTo = useCallback((path: string, options?: { title?: string; description?: string }) => {
    if (options) {
      // Set custom page info before navigation
      const config = PAGE_CONFIG[path];
      setCurrentPage({
        title: options.title || config?.title,
        description: options.description || config?.description
      });
    }
    router.push(path);
  }, [router, setCurrentPage]);

  // Get breadcrumbs from current page
  const breadcrumbs = currentPage?.breadcrumbs || [];

  const value: NavigationContextType = {
    currentPage,
    setCurrentPage,
    history,
    canGoBack,
    canGoForward,
    goBack,
    goForward,
    goHome,
    navigateTo,
    recentPages,
    breadcrumbs
  };

  return (
    <NavigationContext.Provider value={value}>
      {children}
    </NavigationContext.Provider>
  );
}

export function useNavigation() {
  const context = useContext(NavigationContext);
  if (context === undefined) {
    throw new Error('useNavigation must be used within a NavigationProvider');
  }
  return context;
}

// Utility hook for page-specific navigation setup
export function usePageNavigation(title: string, description?: string, customBreadcrumbs?: BreadcrumbItem[]) {
  const { setCurrentPage } = useNavigation();

  useEffect(() => {
    setCurrentPage({
      title,
      description,
      breadcrumbs: customBreadcrumbs
    });
  }, [title, description, customBreadcrumbs, setCurrentPage]);
}