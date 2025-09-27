'use client';

import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { useActiveProcesses } from '@/hooks/use-tauri-api';
import { useNavigation } from '@/contexts/navigation-context';

interface KeyboardShortcutsContextType {
  showExitConfirmation: boolean;
  setShowExitConfirmation: (show: boolean) => void;
  handleEscapeKey: () => void;
}

const KeyboardShortcutsContext = createContext<KeyboardShortcutsContextType | undefined>(undefined);

interface KeyboardShortcutsProviderProps {
  children: ReactNode;
}

export function KeyboardShortcutsProvider({ children }: KeyboardShortcutsProviderProps) {
  const router = useRouter();
  const pathname = usePathname();
  const [showExitConfirmation, setShowExitConfirmation] = useState(false);
  const { data: activeProcesses } = useActiveProcesses();
  const { goBack, goForward, goHome, canGoBack, canGoForward } = useNavigation();

  const isHomePage = pathname === '/';

  const handleEscapeKey = () => {
    if (showExitConfirmation) {
      // If exit confirmation is already shown, close it
      setShowExitConfirmation(false);
      return;
    }

    if (isHomePage) {
      // On home page, show exit confirmation
      setShowExitConfirmation(true);
    } else {
      // On other pages, try to go back first, then home
      if (canGoBack) {
        goBack();
      } else {
        goHome();
      }
    }
  };

  const handleNavigationShortcuts = (event: KeyboardEvent) => {
    // Check for modifier key (Cmd on Mac, Ctrl on Windows/Linux)
    const isModifier = event.metaKey || event.ctrlKey;
    const isAlt = event.altKey;

    // Alt+Arrow navigation shortcuts
    if (isAlt && !isModifier) {
      switch (event.key) {
        case 'ArrowLeft':
          event.preventDefault();
          if (canGoBack) goBack();
          break;
        case 'ArrowRight':
          event.preventDefault();
          if (canGoForward) goForward();
          break;
      }
      return;
    }

    if (!isModifier) return;

    switch (event.key) {
      case '1':
        event.preventDefault();
        router.push('/');
        break;
      case '2':
        event.preventDefault();
        router.push('/models');
        break;
      case '3':
        event.preventDefault();
        router.push('/inference');
        break;
      case '4':
        event.preventDefault();
        router.push('/monitoring');
        break;
      case '5':
        event.preventDefault();
        router.push('/batch');
        break;
      case 'h':
      case 'H':
        event.preventDefault();
        goHome();
        break;
      case 'n':
      case 'N':
        event.preventDefault();
        router.push('/models');
        break;
      case 'r':
      case 'R':
        event.preventDefault();
        router.push('/inference');
        break;
      case 'i':
      case 'I':
        event.preventDefault();
        router.push('/monitoring');
        break;
    }
  };

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      switch (event.key) {
        case 'Escape':
          event.preventDefault();
          handleEscapeKey();
          break;
        default:
          handleNavigationShortcuts(event);
          break;
      }
    };

    document.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isHomePage, showExitConfirmation, router]);

  return (
    <KeyboardShortcutsContext.Provider
      value={{
        showExitConfirmation,
        setShowExitConfirmation,
        handleEscapeKey,
      }}
    >
      {children}
    </KeyboardShortcutsContext.Provider>
  );
}

export function useKeyboardShortcuts() {
  const context = useContext(KeyboardShortcutsContext);
  if (context === undefined) {
    throw new Error('useKeyboardShortcuts must be used within a KeyboardShortcutsProvider');
  }
  return context;
}

// Helper hook to get current active processes info for exit confirmation
export function useExitConfirmationData() {
  const { data: activeProcesses } = useActiveProcesses();

  const hasActiveOperations =
    (activeProcesses?.active_models.length ?? 0) > 0 ||
    (activeProcesses?.active_inferences ?? 0) > 0 ||
    (activeProcesses?.batch_jobs ?? 0) > 0 ||
    (activeProcesses?.streaming_sessions ?? 0) > 0;

  const getActiveOperationsText = () => {
    if (!activeProcesses || !hasActiveOperations) return null;

    const operations = [];

    if (activeProcesses.active_models.length > 0) {
      operations.push(`${activeProcesses.active_models.length} loaded model${activeProcesses.active_models.length > 1 ? 's' : ''}`);
    }

    if (activeProcesses.active_inferences > 0) {
      operations.push(`${activeProcesses.active_inferences} active inference${activeProcesses.active_inferences > 1 ? 's' : ''}`);
    }

    if (activeProcesses.batch_jobs > 0) {
      operations.push(`${activeProcesses.batch_jobs} batch job${activeProcesses.batch_jobs > 1 ? 's' : ''}`);
    }

    if (activeProcesses.streaming_sessions > 0) {
      operations.push(`${activeProcesses.streaming_sessions} streaming session${activeProcesses.streaming_sessions > 1 ? 's' : ''}`);
    }

    return operations.join(', ');
  };

  return {
    hasActiveOperations,
    activeOperationsText: getActiveOperationsText(),
    activeProcesses,
  };
}