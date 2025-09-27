'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ModelUpload } from './model-upload';
import { ModelCard } from './model-card';
import { ModelFilters } from './model-filters';
import { ModelQuantization } from './model-quantization';
import { ModelMarketplace } from './model-marketplace';
import {
  Upload,
  Search,
  Filter,
  Grid,
  List,
  Plus,
  Download,
  RefreshCw,
  Store,
  HardDrive,
} from 'lucide-react';
import { ModelInfo } from '@/types/inferno';
import { useModels, useLoadedModels } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';
import { AlertCircle } from 'lucide-react';

export function ModelManagement() {
  const { data: models = [], isLoading: modelsLoading, error: modelsError, refetch } = useModels();
  const { data: loadedModels = [], isLoading: loadedLoading } = useLoadedModels();
  const [searchQuery, setSearchQuery] = useState('');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [selectedFormat, setSelectedFormat] = useState<string>('all');
  const [selectedStatus, setSelectedStatus] = useState<string>('all');
  const [showUpload, setShowUpload] = useState(false);
  const [showQuantization, setShowQuantization] = useState(false);
  const [selectedModel, setSelectedModel] = useState<ModelInfo | null>(null);

  // Enhance models with loading status
  const enhancedModels = models.map(model => ({
    ...model,
    status: loadedModels.includes(model.id) ? 'loaded' as const : 'available' as const,
    metadata: {
      architecture: 'unknown',
      parameters: 0,
      context_length: 2048,
      created_at: new Date().toISOString(),
      description: `${model.format.toUpperCase()} model file`,
      ...model.metadata,
    },
  })) as ModelInfo[];

  const filteredModels = enhancedModels.filter((model) => {
    const matchesSearch = model.name.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesFormat = selectedFormat === 'all' || model.format === selectedFormat;
    const matchesStatus = selectedStatus === 'all' || model.status === selectedStatus;
    return matchesSearch && matchesFormat && matchesStatus;
  });

  if (modelsLoading || loadedLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">Model Management</h1>
            <p className="text-muted-foreground">Loading models...</p>
          </div>
          <div className="flex items-center space-x-2">
            <Skeleton className="h-6 w-24" />
            <Skeleton className="h-6 w-20" />
            <Skeleton className="h-9 w-32" />
          </div>
        </div>
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className="h-64 w-full" />
          ))}
        </div>
      </div>
    );
  }

  if (modelsError) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">Model Management</h1>
            <p className="text-muted-foreground">Error loading models</p>
          </div>
        </div>
        <Card>
          <CardContent className="flex items-center justify-center py-12">
            <div className="text-center space-y-4">
              <AlertCircle className="h-12 w-12 text-destructive mx-auto" />
              <div>
                <h3 className="text-lg font-semibold text-destructive">Failed to load models</h3>
                <p className="text-muted-foreground">{modelsError.message}</p>
              </div>
              <Button onClick={() => refetch()} variant="outline">
                <RefreshCw className="h-4 w-4 mr-2" />
                Try Again
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Model Management</h1>
          <p className="text-muted-foreground">
            Manage your AI models - discover, download, upload, and monitor performance
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant="secondary" className="flex items-center gap-1">
            {enhancedModels.length} Local Models
          </Badge>
          <Badge variant="success" className="flex items-center gap-1">
            {enhancedModels.filter(m => m.status === 'loaded').length} Active
          </Badge>
        </div>
      </div>

      {/* Main Tabs */}
      <Tabs defaultValue="marketplace" className="space-y-6">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="marketplace" className="flex items-center gap-2">
            <Store className="h-4 w-4" />
            Model Marketplace
          </TabsTrigger>
          <TabsTrigger value="local" className="flex items-center gap-2">
            <HardDrive className="h-4 w-4" />
            Local Models
          </TabsTrigger>
        </TabsList>

        <TabsContent value="marketplace">
          <ModelMarketplace />
        </TabsContent>

        <TabsContent value="local" className="space-y-6">
          {/* Local Models Actions Bar */}
          <Card>
            <CardContent className="p-4">
              <div className="flex items-center justify-between space-x-4">
                {/* Search */}
                <div className="flex-1 max-w-md">
                  <div className="relative">
                    <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <input
                      type="text"
                      placeholder="Search local models..."
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
                  <Button variant="outline" size="sm" onClick={() => refetch()}>
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
                  <Button onClick={() => setShowUpload(true)}>
                    <Upload className="h-4 w-4 mr-2" />
                    Upload Model
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Local Models Grid/List */}
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

          {/* Empty State for Local Models */}
          {filteredModels.length === 0 && (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12">
                <div className="text-center space-y-4">
                  <div className="w-12 h-12 rounded-full bg-muted flex items-center justify-center mx-auto">
                    <Search className="h-6 w-6 text-muted-foreground" />
                  </div>
                  <div>
                    <h3 className="text-lg font-medium">No local models found</h3>
                    <p className="text-muted-foreground">
                      {searchQuery || selectedFormat !== 'all' || selectedStatus !== 'all'
                        ? 'Try adjusting your search criteria or filters'
                        : 'Get started by uploading your first model or download from the marketplace'}
                    </p>
                  </div>
                  {!searchQuery && selectedFormat === 'all' && selectedStatus === 'all' && (
                    <div className="flex gap-2">
                      <Button onClick={() => setShowUpload(true)}>
                        <Plus className="h-4 w-4 mr-2" />
                        Upload Model
                      </Button>
                      <Button variant="outline" onClick={() => {
                        const tabsTrigger = document.querySelector('[value="marketplace"]') as HTMLButtonElement;
                        tabsTrigger?.click();
                      }}>
                        <Store className="h-4 w-4 mr-2" />
                        Browse Marketplace
                      </Button>
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>
      </Tabs>

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