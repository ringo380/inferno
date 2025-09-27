'use client';

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { useKeyboardShortcuts, useExitConfirmationData } from '@/components/providers/keyboard-shortcuts';
import { AlertTriangle, Power } from 'lucide-react';
import { Badge } from '@/components/ui/badge';

export function ExitConfirmation() {
  const { showExitConfirmation, setShowExitConfirmation } = useKeyboardShortcuts();
  const { hasActiveOperations, activeOperationsText } = useExitConfirmationData();

  const handleExit = () => {
    // Close the application
    if (typeof window !== 'undefined' && 'close' in window) {
      window.close();
    } else if (typeof window !== 'undefined' && '__TAURI__' in window) {
      // Tauri-specific exit - fallback to window.close for now
      (window as any).close();
    }
  };

  const handleCancel = () => {
    setShowExitConfirmation(false);
  };

  return (
    <AlertDialog open={showExitConfirmation} onOpenChange={setShowExitConfirmation}>
      <AlertDialogContent className="max-w-md">
        <AlertDialogHeader>
          <div className="flex items-center gap-2">
            <Power className="h-5 w-5 text-destructive" />
            <AlertDialogTitle>Exit Inferno Dashboard</AlertDialogTitle>
          </div>
          <AlertDialogDescription className="space-y-3">
            <p>Are you sure you want to exit the application?</p>

            {hasActiveOperations && (
              <div className="p-3 bg-orange-50 dark:bg-orange-950/20 border border-orange-200 dark:border-orange-800 rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <AlertTriangle className="h-4 w-4 text-orange-600 dark:text-orange-400" />
                  <span className="text-sm font-medium text-orange-800 dark:text-orange-200">
                    Active Operations Detected
                  </span>
                </div>
                <p className="text-sm text-orange-700 dark:text-orange-300 mb-2">
                  The following operations are currently active and will be stopped:
                </p>
                <Badge variant="outline" className="text-xs bg-orange-100 dark:bg-orange-900/30 border-orange-300 dark:border-orange-700">
                  {activeOperationsText}
                </Badge>
              </div>
            )}

            <p className="text-sm text-muted-foreground">
              {hasActiveOperations
                ? 'All active operations will be gracefully stopped before exiting.'
                : 'This action cannot be undone.'
              }
            </p>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={handleCancel}>
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={handleExit}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            <Power className="h-4 w-4 mr-2" />
            Exit Application
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}