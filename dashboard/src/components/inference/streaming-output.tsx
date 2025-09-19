'use client';

import { useEffect, useRef } from 'react';
import { Badge } from '@/components/ui/badge';
import { Copy, Download } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { copyToClipboard } from '@/lib/utils';
import toast from 'react-hot-toast';

interface StreamingOutputProps {
  response: string;
  isStreaming: boolean;
  metrics: {
    tokensGenerated: number;
    inferenceTime: number;
    tokensPerSecond: number;
  };
}

export function StreamingOutput({ response, isStreaming, metrics }: StreamingOutputProps) {
  const outputRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new content arrives
  useEffect(() => {
    if (outputRef.current) {
      outputRef.current.scrollTop = outputRef.current.scrollHeight;
    }
  }, [response]);

  const handleCopy = async () => {
    try {
      await copyToClipboard(response);
      toast.success('Response copied to clipboard');
    } catch (error) {
      toast.error('Failed to copy response');
    }
  };

  const handleDownload = () => {
    const blob = new Blob([response], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `inference-response-${Date.now()}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    toast.success('Response downloaded');
  };

  return (
    <div className="space-y-4">
      {/* Output Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          {isStreaming && (
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
              <span className="text-sm text-muted-foreground">Generating...</span>
            </div>
          )}
          {!isStreaming && response && (
            <Badge variant="success" className="text-xs">
              Generation Complete
            </Badge>
          )}
        </div>

        {response && (
          <div className="flex items-center space-x-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleCopy}
            >
              <Copy className="h-4 w-4 mr-2" />
              Copy
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleDownload}
            >
              <Download className="h-4 w-4 mr-2" />
              Download
            </Button>
          </div>
        )}
      </div>

      {/* Output Content */}
      <div
        ref={outputRef}
        className="min-h-[200px] max-h-[400px] p-4 border rounded-lg bg-muted/30 overflow-y-auto scrollbar-thin"
      >
        {response ? (
          <div className="whitespace-pre-wrap text-sm font-mono leading-relaxed">
            {response}
            {isStreaming && (
              <span className="inline-block w-2 h-5 bg-primary ml-1 animate-pulse" />
            )}
          </div>
        ) : (
          <div className="flex items-center justify-center h-[200px] text-muted-foreground">
            <div className="text-center">
              <div className="text-lg mb-2">💭</div>
              <p>Response will appear here when you run inference</p>
            </div>
          </div>
        )}
      </div>

      {/* Metrics Bar */}
      {(response || isStreaming) && (
        <div className="grid grid-cols-3 gap-4 p-3 bg-muted/50 rounded-lg text-sm">
          <div className="text-center">
            <div className="text-muted-foreground text-xs">Generation Time</div>
            <div className="font-medium">
              {(metrics.inferenceTime / 1000).toFixed(1)}s
            </div>
          </div>
          <div className="text-center">
            <div className="text-muted-foreground text-xs">Tokens/Second</div>
            <div className="font-medium">
              {metrics.tokensPerSecond.toFixed(1)}
            </div>
          </div>
          <div className="text-center">
            <div className="text-muted-foreground text-xs">Tokens Generated</div>
            <div className="font-medium">
              {metrics.tokensGenerated}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}