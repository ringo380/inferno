'use client';

import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Filter, X } from 'lucide-react';
import { useState } from 'react';

interface ModelFiltersProps {
  selectedFormat: string;
  selectedStatus: string;
  onFormatChange: (format: string) => void;
  onStatusChange: (status: string) => void;
}

const formatOptions = [
  { value: 'all', label: 'All Formats', count: 12 },
  { value: 'gguf', label: 'GGUF', count: 7 },
  { value: 'onnx', label: 'ONNX', count: 3 },
  { value: 'pytorch', label: 'PyTorch', count: 1 },
  { value: 'safetensors', label: 'SafeTensors', count: 1 },
];

const statusOptions = [
  { value: 'all', label: 'All Status', count: 12 },
  { value: 'loaded', label: 'Loaded', count: 3 },
  { value: 'available', label: 'Available', count: 7 },
  { value: 'loading', label: 'Loading', count: 1 },
  { value: 'error', label: 'Error', count: 1 },
];

export function ModelFilters({
  selectedFormat,
  selectedStatus,
  onFormatChange,
  onStatusChange,
}: ModelFiltersProps) {
  const [showFilters, setShowFilters] = useState(false);

  const activeFiltersCount = (selectedFormat !== 'all' ? 1 : 0) + (selectedStatus !== 'all' ? 1 : 0);

  const clearFilters = () => {
    onFormatChange('all');
    onStatusChange('all');
  };

  return (
    <div className="relative">
      <Button
        variant="outline"
        size="sm"
        onClick={() => setShowFilters(!showFilters)}
        className="relative"
      >
        <Filter className="h-4 w-4 mr-2" />
        Filters
        {activeFiltersCount > 0 && (
          <Badge
            variant="secondary"
            className="ml-2 h-5 text-xs"
          >
            {activeFiltersCount}
          </Badge>
        )}
      </Button>

      {showFilters && (
        <div className="absolute top-full right-0 mt-2 w-80 bg-background border rounded-lg shadow-lg z-50">
          <div className="p-4 space-y-4">
            {/* Header */}
            <div className="flex items-center justify-between">
              <h3 className="font-medium">Filter Models</h3>
              <div className="flex items-center space-x-2">
                {activeFiltersCount > 0 && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={clearFilters}
                    className="text-xs"
                  >
                    Clear All
                  </Button>
                )}
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setShowFilters(false)}
                >
                  <X className="h-4 w-4" />
                </Button>
              </div>
            </div>

            {/* Format Filter */}
            <div>
              <label className="text-sm font-medium text-muted-foreground">
                Format
              </label>
              <div className="mt-2 space-y-1">
                {formatOptions.map((option) => (
                  <button
                    key={option.value}
                    onClick={() => onFormatChange(option.value)}
                    className={`w-full flex items-center justify-between p-2 text-sm rounded-md hover:bg-accent ${
                      selectedFormat === option.value
                        ? 'bg-accent text-accent-foreground'
                        : 'text-muted-foreground'
                    }`}
                  >
                    <span>{option.label}</span>
                    <Badge variant="secondary" className="text-xs">
                      {option.count}
                    </Badge>
                  </button>
                ))}
              </div>
            </div>

            {/* Status Filter */}
            <div>
              <label className="text-sm font-medium text-muted-foreground">
                Status
              </label>
              <div className="mt-2 space-y-1">
                {statusOptions.map((option) => (
                  <button
                    key={option.value}
                    onClick={() => onStatusChange(option.value)}
                    className={`w-full flex items-center justify-between p-2 text-sm rounded-md hover:bg-accent ${
                      selectedStatus === option.value
                        ? 'bg-accent text-accent-foreground'
                        : 'text-muted-foreground'
                    }`}
                  >
                    <span>{option.label}</span>
                    <Badge variant="secondary" className="text-xs">
                      {option.count}
                    </Badge>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}