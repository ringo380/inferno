'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ModelUpload } from './model-upload';
import { ModelCard } from './model-card';
import { ModelFilters } from './model-filters';
import { ModelQuantization } from './model-quantization';
import {
  Upload,
  Search,
  Filter,
  Grid,
  List,
  Plus,
  Download,
  RefreshCw,
} from 'lucide-react';
import { ModelInfo } from '@/types/inferno';

const mockModels: ModelInfo[] = [
  {
    id: 'llama-7b-q4',
    name: 'Llama 2 7B Q4_0',
    path: '/models/llama-2-7b-chat.q4_0.gguf',
    format: 'gguf',
    size: 3800000000,
    checksum: 'sha256:abc123...',
    status: 'loaded',
    metadata: {
      architecture: 'llama',
      parameters: 7000000000,
      quantization: 'Q4_0',
      context_length: 4096,
      created_at: '2024-01-15T10:30:00Z',
      description: 'Llama 2 7B model optimized for chat applications',
    },
  },
  {
    id: 'gpt-3.5-turbo',
    name: 'GPT-3.5 Turbo',
    path: '/models/gpt-3.5-turbo.onnx',
    format: 'onnx',
    size: 6400000000,
    checksum: 'sha256:def456...',
    status: 'available',
    metadata: {
      architecture: 'transformer',
      parameters: 175000000000,
      context_length: 4096,
      created_at: '2024-01-10T14:20:00Z',
      description: 'OpenAI GPT-3.5 Turbo model for general purpose tasks',
    },
  },
  {
    id: 'mistral-7b-instruct',
    name: 'Mistral 7B Instruct',
    path: '/models/mistral-7b-instruct.gguf',
    format: 'gguf',
    size: 4200000000,
    checksum: 'sha256:ghi789...',
    status: 'error',
    metadata: {
      architecture: 'mistral',
      parameters: 7000000000,
      quantization: 'F16',
      context_length: 8192,
      created_at: '2024-01-12T09:15:00Z',
      description: 'Mistral 7B model fine-tuned for instruction following',
    },
  },
  {
    id: 'claude-instant',
    name: 'Claude Instant',
    path: '/models/claude-instant.onnx',
    format: 'onnx',
    size: 8500000000,
    checksum: 'sha256:jkl012...',
    status: 'loading',
    metadata: {
      architecture: 'claude',
      parameters: 52000000000,
      context_length: 100000,
      created_at: '2024-01-18T16:45:00Z',
      description: 'Anthropic Claude Instant model for fast, helpful responses',
    },
  },
];

export function ModelManagement() {
  const [searchQuery, setSearchQuery] = useState('');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [selectedFormat, setSelectedFormat] = useState<string>('all');
  const [selectedStatus, setSelectedStatus] = useState<string>('all');
  const [showUpload, setShowUpload] = useState(false);
  const [showQuantization, setShowQuantization] = useState(false);
  const [selectedModel, setSelectedModel] = useState<ModelInfo | null>(null);

  const filteredModels = mockModels.filter((model) => {
    const matchesSearch = model.name.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesFormat = selectedFormat === 'all' || model.format === selectedFormat;
    const matchesStatus = selectedStatus === 'all' || model.status === selectedStatus;
    return matchesSearch && matchesFormat && matchesStatus;
  });

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Model Management</h1>
          <p className="text-muted-foreground">
            Manage your AI models - upload, quantize, and monitor performance
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant="secondary" className="flex items-center gap-1">
            {mockModels.length} Total Models
          </Badge>
          <Badge variant="success" className="flex items-center gap-1">
            {mockModels.filter(m => m.status === 'loaded').length} Active
          </Badge>
          <Button onClick={() => setShowUpload(true)}>
            <Upload className="h-4 w-4 mr-2" />
            Upload Model
          </Button>
        </div>
      </div>

      {/* Actions Bar */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center justify-between space-x-4">
            {/* Search */}
            <div className="flex-1 max-w-md">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <input
                  type="text"
                  placeholder="Search models..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                />
              </div>
            </div>

            {/* Filters */}
            <ModelFilters
              selectedFormat={selectedFormat}
              selectedStatus={selectedStatus}
              onFormatChange={setSelectedFormat}
              onStatusChange={setSelectedStatus}
            />

            {/* View Mode Toggle */}
            <div className="flex items-center space-x-1 border rounded-md p-1">
              <Button
                variant={viewMode === 'grid' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setViewMode('grid')}
              >
                <Grid className="h-4 w-4" />
              </Button>
              <Button
                variant={viewMode === 'list' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setViewMode('list')}
              >
                <List className="h-4 w-4" />
              </Button>
            </div>

            {/* Actions */}
            <div className="flex items-center space-x-2">
              <Button variant="outline" size="sm">
                <RefreshCw className="h-4 w-4 mr-2" />
                Refresh
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowQuantization(true)}
              >
                <Download className="h-4 w-4 mr-2" />
                Quantize
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Models Grid/List */}
      <div className={
        viewMode === 'grid'
          ? 'grid gap-6 md:grid-cols-2 lg:grid-cols-3'
          : 'space-y-4'
      }>
        {filteredModels.map((model) => (
          <ModelCard
            key={model.id}
            model={model}
            viewMode={viewMode}
            onSelect={setSelectedModel}
            onQuantize={() => {
              setSelectedModel(model);
              setShowQuantization(true);
            }}
          />
        ))}
      </div>

      {/* Empty State */}
      {filteredModels.length === 0 && (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <div className="text-center space-y-4">
              <div className="w-12 h-12 rounded-full bg-muted flex items-center justify-center mx-auto">
                <Search className="h-6 w-6 text-muted-foreground" />
              </div>
              <div>
                <h3 className="text-lg font-medium">No models found</h3>
                <p className="text-muted-foreground">
                  {searchQuery || selectedFormat !== 'all' || selectedStatus !== 'all'
                    ? 'Try adjusting your search criteria or filters'
                    : 'Get started by uploading your first model'}
                </p>
              </div>
              {!searchQuery && selectedFormat === 'all' && selectedStatus === 'all' && (
                <Button onClick={() => setShowUpload(true)}>
                  <Plus className="h-4 w-4 mr-2" />
                  Upload First Model
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Upload Modal */}
      {showUpload && (
        <ModelUpload
          onClose={() => setShowUpload(false)}
          onUpload={(file) => {
            console.log('Uploading:', file);
            setShowUpload(false);
          }}
        />
      )}

      {/* Quantization Modal */}
      {showQuantization && selectedModel && (
        <ModelQuantization
          model={selectedModel}
          onClose={() => {
            setShowQuantization(false);
            setSelectedModel(null);
          }}
          onQuantize={(options) => {
            console.log('Quantizing:', selectedModel, options);
            setShowQuantization(false);
            setSelectedModel(null);
          }}
        />
      )}
    </div>
  );
}