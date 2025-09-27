'use client';

import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';
import { Button } from './button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './card';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);

    // Call the optional onError callback
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }

    // In a real application, you would send this to an error reporting service
    // Example: Sentry.captureException(error, { contexts: { errorInfo } });
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: undefined });
  };

  render() {
    if (this.state.hasError) {
      // Custom fallback UI
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default fallback UI
      return (
        <Card className="border-destructive">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertTriangle className="h-5 w-5" />
              Something went wrong
            </CardTitle>
            <CardDescription>
              An unexpected error occurred. Please try refreshing the component.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {this.state.error && (
              <details className="text-sm bg-muted p-3 rounded-md">
                <summary className="cursor-pointer font-medium mb-2">
                  Error Details
                </summary>
                <pre className="whitespace-pre-wrap text-xs">
                  {this.state.error.message}
                  {'\n\n'}
                  {this.state.error.stack}
                </pre>
              </details>
            )}
            <Button onClick={this.handleRetry} className="flex items-center gap-2">
              <RefreshCw className="h-4 w-4" />
              Try Again
            </Button>
          </CardContent>
        </Card>
      );
    }

    return this.props.children;
  }
}

// Hook version for functional components
export function useErrorHandler() {
  return React.useCallback((error: Error, errorInfo?: ErrorInfo) => {
    console.error('Error caught by useErrorHandler:', error, errorInfo);

    // In a real application, you would send this to an error reporting service
    // Example: Sentry.captureException(error, { contexts: { errorInfo } });
  }, []);
}

// Higher-order component version
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  fallback?: ReactNode,
  onError?: (error: Error, errorInfo: ErrorInfo) => void
) {
  const WrappedComponent = (props: P) => (
    <ErrorBoundary fallback={fallback} onError={onError}>
      <Component {...props} />
    </ErrorBoundary>
  );

  WrappedComponent.displayName = `withErrorBoundary(${Component.displayName || Component.name})`;

  return WrappedComponent;
}