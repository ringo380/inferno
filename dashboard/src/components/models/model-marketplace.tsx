'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Search, Download, Star, Users, Calendar, Tag, ExternalLink } from 'lucide-react';
import { LoadingSpinner } from '@/components/ui/loading-spinner';

interface ModelFileInfo {
  filename: string;
  size_bytes: number;
  download_url: string;
  file_type: string;
}

interface ExternalModelInfo {
  id: string;
  name: string;
  author: string;
  description: string;
  tags: string[];
  model_type: string;
  size_bytes?: number;
  download_url: string;
  repository_url: string;
  license: string;
  downloads: number;
  likes: number;
  created_at: string;
  updated_at: string;
  file_info: ModelFileInfo[];
}

interface ModelSearchResponse {
  models: ExternalModelInfo[];
  total: number;
  has_more: boolean;
}

interface DownloadProgress {
  download_id: string;
  model_id: string;
  filename: string;
  downloaded_bytes: number;
  total_bytes: number;
  progress_percent: number;
  status: string;
  error_message?: string;
  download_speed_bps?: number;
  eta_seconds?: number;
  started_at: string;
  completed_at?: string;
}

export function ModelMarketplace() {
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<ExternalModelInfo[]>([]);
  const [featuredModels, setFeaturedModels] = useState<ExternalModelInfo[]>([]);
  const [trendingModels, setTrendingModels] = useState<ExternalModelInfo[]>([]);
  const [downloads, setDownloads] = useState<DownloadProgress[]>([]);
  const [loading, setLoading] = useState({
    search: false,
    featured: false,
    trending: false,
  });
  const [activeTab, setActiveTab] = useState('featured');

  useEffect(() => {
    loadFeaturedModels();
    loadTrendingModels();
    loadDownloads();

    // Poll for download progress updates
    const interval = setInterval(loadDownloads, 2000);
    return () => clearInterval(interval);
  }, []);

  const loadFeaturedModels = async () => {
    setLoading(prev => ({ ...prev, featured: true }));
    try {
      // Mock featured models for browser mode
      const models: ExternalModelInfo[] = [
        {
          id: 'microsoft/DialoGPT-medium',
          name: 'DialoGPT Medium',
          author: 'Microsoft',
          description: 'A medium-scale neural conversational response generation model',
          tags: ['conversational', 'chatbot', 'transformers'],
          model_type: 'text-generation',
          size_bytes: 354000000,
          download_url: 'https://huggingface.co/microsoft/DialoGPT-medium',
          repository_url: 'https://huggingface.co/microsoft/DialoGPT-medium',
          license: 'MIT',
          downloads: 145230,
          likes: 892,
          created_at: '2023-01-15T10:30:00Z',
          updated_at: '2024-01-15T10:30:00Z',
          file_info: []
        },
        {
          id: 'meta-llama/Llama-2-7b-chat-hf',
          name: 'Llama 2 7B Chat',
          author: 'Meta',
          description: 'A 7 billion parameter conversational model fine-tuned for chat use cases',
          tags: ['llama', 'chat', 'meta', 'conversational'],
          model_type: 'text-generation',
          size_bytes: 13000000000,
          download_url: 'https://huggingface.co/meta-llama/Llama-2-7b-chat-hf',
          repository_url: 'https://huggingface.co/meta-llama/Llama-2-7b-chat-hf',
          license: 'Custom',
          downloads: 298450,
          likes: 1547,
          created_at: '2023-07-18T14:20:00Z',
          updated_at: '2024-02-10T14:20:00Z',
          file_info: []
        },
        {
          id: 'sentence-transformers/all-MiniLM-L6-v2',
          name: 'All MiniLM L6 v2',
          author: 'Sentence Transformers',
          description: 'Sentence embedding model for semantic similarity',
          tags: ['sentence-similarity', 'embeddings', 'transformers'],
          model_type: 'feature-extraction',
          size_bytes: 90000000,
          download_url: 'https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2',
          repository_url: 'https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2',
          license: 'Apache-2.0',
          downloads: 523847,
          likes: 2103,
          created_at: '2022-08-05T09:15:00Z',
          updated_at: '2023-11-20T09:15:00Z',
          file_info: []
        }
      ];
      setFeaturedModels(models);
    } catch (error) {
      console.error('Failed to load featured models:', error);
    } finally {
      setLoading(prev => ({ ...prev, featured: false }));
    }
  };

  const loadTrendingModels = async () => {
    setLoading(prev => ({ ...prev, trending: true }));
    try {
      // Mock trending models for browser mode
      const models: ExternalModelInfo[] = [
        {
          id: 'openai/whisper-large-v3',
          name: 'Whisper Large v3',
          author: 'OpenAI',
          description: 'Automatic speech recognition model with improved accuracy',
          tags: ['speech-recognition', 'audio', 'whisper'],
          model_type: 'automatic-speech-recognition',
          size_bytes: 1550000000,
          download_url: 'https://huggingface.co/openai/whisper-large-v3',
          repository_url: 'https://huggingface.co/openai/whisper-large-v3',
          license: 'MIT',
          downloads: 187234,
          likes: 934,
          created_at: '2023-11-01T16:45:00Z',
          updated_at: '2024-01-25T16:45:00Z',
          file_info: []
        },
        {
          id: 'stabilityai/stable-diffusion-xl-base-1.0',
          name: 'Stable Diffusion XL',
          author: 'Stability AI',
          description: 'Text-to-image diffusion model with enhanced resolution',
          tags: ['text-to-image', 'diffusion', 'generative'],
          model_type: 'text-to-image',
          size_bytes: 6900000000,
          download_url: 'https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0',
          repository_url: 'https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0',
          license: 'CreativeML Open RAIL++-M',
          downloads: 432167,
          likes: 2876,
          created_at: '2023-07-26T11:30:00Z',
          updated_at: '2024-01-18T11:30:00Z',
          file_info: []
        }
      ];
      setTrendingModels(models);
    } catch (error) {
      console.error('Failed to load trending models:', error);
    } finally {
      setLoading(prev => ({ ...prev, trending: false }));
    }
  };

  const loadDownloads = async () => {
    try {
      // Mock downloads for browser mode
      const allDownloads: DownloadProgress[] = [];
      setDownloads(allDownloads);
    } catch (error) {
      console.error('Failed to load downloads:', error);
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;

    setLoading(prev => ({ ...prev, search: true }));
    try {
      // Mock search results for browser mode
      const mockResults: ExternalModelInfo[] = [
        {
          id: 'bert-base-uncased',
          name: 'BERT Base Uncased',
          author: 'Google',
          description: 'Bidirectional Encoder Representations from Transformers',
          tags: ['bert', 'language-model', 'transformers'],
          model_type: 'fill-mask',
          size_bytes: 440000000,
          download_url: 'https://huggingface.co/bert-base-uncased',
          repository_url: 'https://huggingface.co/bert-base-uncased',
          license: 'Apache-2.0',
          downloads: 789234,
          likes: 3456,
          created_at: '2020-05-15T10:30:00Z',
          updated_at: '2023-08-20T10:30:00Z',
          file_info: []
        }
      ].filter(model =>
        model.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        model.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        model.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()))
      );

      setSearchResults(mockResults);
      setActiveTab('search');
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setLoading(prev => ({ ...prev, search: false }));
    }
  };

  const handleDownload = async (model: ExternalModelInfo) => {
    try {
      // Mock download for browser mode
      console.log('Download started for model:', model.name);
      alert(`Download started for ${model.name}. In Tauri mode, this would actually download the model.`);
    } catch (error) {
      console.error('Failed to start download:', error);
    }
  };

  const cancelDownload = async (downloadId: string) => {
    try {
      console.log('Download cancelled:', downloadId);
      loadDownloads();
    } catch (error) {
      console.error('Failed to cancel download:', error);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatSpeed = (bps?: number) => {
    if (!bps) return 'Unknown';
    return formatBytes(bps) + '/s';
  };

  const formatETA = (seconds?: number) => {
    if (!seconds) return 'Unknown';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) return `${hours}h ${minutes}m`;
    if (minutes > 0) return `${minutes}m ${secs}s`;
    return `${secs}s`;
  };

  const ModelCard = ({ model }: { model: ExternalModelInfo }) => {
    const isDownloading = downloads.some(d => d.model_id === model.id && d.status === 'downloading');
    const downloadProgress = downloads.find(d => d.model_id === model.id);

    return (
      <Card className="h-full">
        <CardHeader>
          <div className="flex justify-between items-start">
            <div className="flex-1">
              <CardTitle className="text-lg font-semibold mb-1">{model.name}</CardTitle>
              <CardDescription className="text-sm text-muted-foreground">
                by {model.author}
              </CardDescription>
            </div>
            <Badge variant="secondary" className="ml-2">
              {model.model_type}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground mb-4 line-clamp-3">
            {model.description || 'No description available'}
          </p>

          <div className="flex flex-wrap gap-1 mb-4">
            {model.tags.slice(0, 3).map((tag, index) => (
              <Badge key={index} variant="outline" className="text-xs">
                <Tag className="w-3 h-3 mr-1" />
                {tag}
              </Badge>
            ))}
            {model.tags.length > 3 && (
              <Badge variant="outline" className="text-xs">
                +{model.tags.length - 3} more
              </Badge>
            )}
          </div>

          <div className="grid grid-cols-2 gap-4 mb-4 text-sm">
            <div className="flex items-center">
              <Download className="w-4 h-4 mr-1 text-muted-foreground" />
              <span>{model.downloads.toLocaleString()}</span>
            </div>
            <div className="flex items-center">
              <Star className="w-4 h-4 mr-1 text-muted-foreground" />
              <span>{model.likes.toLocaleString()}</span>
            </div>
            <div className="flex items-center">
              <Users className="w-4 h-4 mr-1 text-muted-foreground" />
              <span>{model.license}</span>
            </div>
            <div className="flex items-center">
              <Calendar className="w-4 h-4 mr-1 text-muted-foreground" />
              <span>{new Date(model.updated_at).toLocaleDateString()}</span>
            </div>
          </div>

          {model.size_bytes && (
            <div className="text-sm text-muted-foreground mb-4">
              Size: {formatBytes(model.size_bytes)}
            </div>
          )}

          {downloadProgress && (
            <div className="mb-4">
              <div className="flex justify-between text-sm mb-2">
                <span>Downloading: {downloadProgress.status}</span>
                <span>{downloadProgress.progress_percent.toFixed(1)}%</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${downloadProgress.progress_percent}%` }}
                />
              </div>
              <div className="flex justify-between text-xs text-muted-foreground mt-1">
                <span>Speed: {formatSpeed(downloadProgress.download_speed_bps)}</span>
                <span>ETA: {formatETA(downloadProgress.eta_seconds)}</span>
              </div>
            </div>
          )}

          <div className="flex gap-2">
            <Button
              onClick={() => handleDownload(model)}
              disabled={isDownloading}
              className="flex-1"
            >
              {isDownloading ? (
                <>
                  <LoadingSpinner className="w-4 h-4 mr-2" />
                  Downloading...
                </>
              ) : (
                <>
                  <Download className="w-4 h-4 mr-2" />
                  Download
                </>
              )}
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => window.open(model.repository_url, '_blank')}
            >
              <ExternalLink className="w-4 h-4" />
            </Button>
          </div>

          {downloadProgress && downloadProgress.status === 'downloading' && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => cancelDownload(downloadProgress.download_id)}
              className="w-full mt-2"
            >
              Cancel Download
            </Button>
          )}
        </CardContent>
      </Card>
    );
  };

  return (
    <div className="space-y-6">
      {/* Search Bar */}
      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
          <Input
            placeholder="Search for models (e.g., 'llama', 'bert', 'gpt')..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
            className="pl-10"
          />
        </div>
        <Button onClick={handleSearch} disabled={loading.search}>
          {loading.search ? <LoadingSpinner className="w-4 h-4" /> : 'Search'}
        </Button>
      </div>

      {/* Active Downloads */}
      {downloads.filter(d => d.status === 'downloading').length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Active Downloads</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {downloads
                .filter(d => d.status === 'downloading')
                .map((download) => (
                  <div key={download.download_id} className="flex items-center space-x-4">
                    <div className="flex-1">
                      <div className="flex justify-between text-sm mb-1">
                        <span className="font-medium">{download.filename}</span>
                        <span>{download.progress_percent.toFixed(1)}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                          style={{ width: `${download.progress_percent}%` }}
                        />
                      </div>
                      <div className="flex justify-between text-xs text-muted-foreground mt-1">
                        <span>{formatBytes(download.downloaded_bytes)} / {formatBytes(download.total_bytes)}</span>
                        <span>Speed: {formatSpeed(download.download_speed_bps)}</span>
                      </div>
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => cancelDownload(download.download_id)}
                    >
                      Cancel
                    </Button>
                  </div>
                ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Model Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="featured">Featured Models</TabsTrigger>
          <TabsTrigger value="trending">Trending Models</TabsTrigger>
          <TabsTrigger value="search">Search Results</TabsTrigger>
        </TabsList>

        <TabsContent value="featured" className="space-y-4">
          {loading.featured ? (
            <div className="flex justify-center py-8">
              <LoadingSpinner className="w-8 h-8" />
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {featuredModels.map((model) => (
                <ModelCard key={model.id} model={model} />
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="trending" className="space-y-4">
          {loading.trending ? (
            <div className="flex justify-center py-8">
              <LoadingSpinner className="w-8 h-8" />
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {trendingModels.map((model) => (
                <ModelCard key={model.id} model={model} />
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="search" className="space-y-4">
          {loading.search ? (
            <div className="flex justify-center py-8">
              <LoadingSpinner className="w-8 h-8" />
            </div>
          ) : searchResults.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {searchResults.map((model) => (
                <ModelCard key={model.id} model={model} />
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              {searchQuery ? 'No models found for your search.' : 'Search for models to see results here.'}
            </div>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}