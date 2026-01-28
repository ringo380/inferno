'use client';

import { useState, useEffect } from 'react';
import { MainLayout } from '@/components/layout/main-layout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Switch } from '@/components/ui/switch';
import { Slider } from '@/components/ui/slider';
import { Separator } from '@/components/ui/separator';
import {
  Settings,
  Folder,
  Cpu,
  HardDrive,
  Brain,
  Zap,
  Shield,
  Database,
  Globe,
  Save,
  RotateCcw,
  CheckCircle,
  AlertTriangle,
  Info,
} from 'lucide-react';
import { toast } from 'react-hot-toast';
import { useSettings, useUpdateSettings } from '@/hooks/use-tauri-api';
import { AppSettings } from '@/types/inferno';

const defaultSettings: AppSettings = {
  modelsDirectory: 'test_models/test_models',
  autoDiscoverModels: true,
  defaultTemperature: 0.7,
  defaultMaxTokens: 512,
  defaultTopP: 0.9,
  defaultTopK: 40,
  maxMemoryUsage: 80,
  preferGPU: true,
  maxConcurrentInferences: 3,
  enableCache: true,
  cacheDirectory: '.cache',
  maxCacheSize: 1024,
  enableRestAPI: false,
  apiPort: 8080,
  enableCORS: true,
  requireAuthentication: false,
  enableAuditLog: true,
  logLevel: 'info',
  notifications: {
    enabled: true,
    types: {
      system: true,
      inference: true,
      security: true,
      batch: true,
      model: true,
    },
    priority_filter: 'all',
    auto_dismiss_after: 10,
    max_notifications: 50,
  },
};

export default function SettingsPage() {
  const { data: backendSettings, isLoading, error } = useSettings();
  const updateSettingsMutation = useUpdateSettings();

  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [isDirty, setIsDirty] = useState(false);

  // Load settings from backend when available
  useEffect(() => {
    if (backendSettings) {
      setSettings(backendSettings);
    }
  }, [backendSettings]);

  const updateSetting = (key: keyof AppSettings, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    setIsDirty(true);
  };

  const handleSave = async () => {
    try {
      await updateSettingsMutation.mutateAsync(settings);
      setIsDirty(false);
    } catch (error) {
      // Error is already handled by the mutation
      console.error('Failed to save settings:', error);
    }
  };

  const handleReset = () => {
    setSettings(defaultSettings);
    setIsDirty(true);
    toast('Settings reset to defaults');
  };

  if (isLoading) {
    return (
      <MainLayout>
        <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
            <p className="text-muted-foreground">Loading settings...</p>
          </div>
        </div>
        <div className="grid gap-6">
          {[...Array(6)].map((_, i) => (
            <Card key={i}>
              <CardHeader>
                <div className="h-6 w-48 bg-muted animate-pulse rounded" />
                <div className="h-4 w-96 bg-muted animate-pulse rounded" />
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="h-4 w-32 bg-muted animate-pulse rounded" />
                  <div className="h-8 w-full bg-muted animate-pulse rounded" />
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
        </div>
      </MainLayout>
    );
  }

  if (error) {
    return (
      <MainLayout>
        <div className="space-y-6">
        <Card className="border-destructive">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertTriangle className="h-5 w-5" />
              Error Loading Settings
            </CardTitle>
            <CardDescription>
              Failed to load application settings. Please check your connection.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              {error instanceof Error ? error.message : 'Unknown error occurred'}
            </p>
          </CardContent>
        </Card>
        </div>
      </MainLayout>
    );
  }

  return (
    <MainLayout>
      <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
          <p className="text-muted-foreground">
            Configure Inferno application preferences and behavior
          </p>
        </div>
        <div className="flex items-center space-x-2">
          {isDirty && (
            <Badge variant="warning" className="flex items-center gap-1">
              <AlertTriangle className="h-3 w-3" />
              Unsaved Changes
            </Badge>
          )}
          <Button variant="outline" onClick={handleReset}>
            <RotateCcw className="h-4 w-4 mr-2" />
            Reset
          </Button>
          <Button onClick={handleSave} disabled={!isDirty || updateSettingsMutation.isPending}>
            <Save className="h-4 w-4 mr-2" />
            {updateSettingsMutation.isPending ? 'Saving...' : 'Save Changes'}
          </Button>
        </div>
      </div>

      {/* Model Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Brain className="h-5 w-5" />
            Model Settings
          </CardTitle>
          <CardDescription>
            Configure model discovery and default behavior
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <label className="text-sm font-medium">Models Directory</label>
            <div className="flex items-center space-x-2">
              <input
                type="text"
                value={settings.modelsDirectory}
                onChange={(e) => updateSetting('modelsDirectory', e.target.value)}
                className="flex-1 px-3 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500"
                placeholder="/path/to/models"
              />
              <Button variant="outline" size="sm">
                <Folder className="h-4 w-4" />
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              Directory where Inferno will look for AI models
            </p>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <label className="text-sm font-medium">Auto-discover Models</label>
              <p className="text-xs text-muted-foreground">
                Automatically scan for new models on startup
              </p>
            </div>
            <Switch
              checked={settings.autoDiscoverModels}
              onCheckedChange={(checked) => updateSetting('autoDiscoverModels', checked)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Inference Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="h-5 w-5" />
            Inference Settings
          </CardTitle>
          <CardDescription>
            Default parameters for model inference
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium">Temperature</label>
              <span className="text-sm text-muted-foreground">{settings.defaultTemperature}</span>
            </div>
            <Slider
              value={[settings.defaultTemperature]}
              onValueChange={([value]) => updateSetting('defaultTemperature', value)}
              max={2}
              min={0}
              step={0.1}
              className="w-full"
            />
            <p className="text-xs text-muted-foreground">
              Controls randomness in model outputs (0 = deterministic, 2 = very random)
            </p>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium">Max Tokens</label>
              <span className="text-sm text-muted-foreground">{settings.defaultMaxTokens}</span>
            </div>
            <Slider
              value={[settings.defaultMaxTokens]}
              onValueChange={([value]) => updateSetting('defaultMaxTokens', value)}
              max={4096}
              min={1}
              step={1}
              className="w-full"
            />
            <p className="text-xs text-muted-foreground">
              Maximum number of tokens to generate in response
            </p>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium">Top P</label>
                <span className="text-sm text-muted-foreground">{settings.defaultTopP}</span>
              </div>
              <Slider
                value={[settings.defaultTopP]}
                onValueChange={([value]) => updateSetting('defaultTopP', value)}
                max={1}
                min={0}
                step={0.01}
                className="w-full"
              />
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium">Top K</label>
                <span className="text-sm text-muted-foreground">{settings.defaultTopK}</span>
              </div>
              <Slider
                value={[settings.defaultTopK]}
                onValueChange={([value]) => updateSetting('defaultTopK', value)}
                max={100}
                min={1}
                step={1}
                className="w-full"
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* System Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Cpu className="h-5 w-5" />
            System Settings
          </CardTitle>
          <CardDescription>
            Performance and resource management settings
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium">Max Memory Usage</label>
              <span className="text-sm text-muted-foreground">{settings.maxMemoryUsage}%</span>
            </div>
            <Slider
              value={[settings.maxMemoryUsage]}
              onValueChange={([value]) => updateSetting('maxMemoryUsage', value)}
              max={95}
              min={10}
              step={5}
              className="w-full"
            />
            <p className="text-xs text-muted-foreground">
              Maximum percentage of system memory that Inferno can use
            </p>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <label className="text-sm font-medium">Prefer GPU</label>
              <p className="text-xs text-muted-foreground">
                Use GPU acceleration when available
              </p>
            </div>
            <Switch
              checked={settings.preferGPU}
              onCheckedChange={(checked) => updateSetting('preferGPU', checked)}
            />
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium">Max Concurrent Inferences</label>
              <span className="text-sm text-muted-foreground">{settings.maxConcurrentInferences}</span>
            </div>
            <Slider
              value={[settings.maxConcurrentInferences]}
              onValueChange={([value]) => updateSetting('maxConcurrentInferences', value)}
              max={10}
              min={1}
              step={1}
              className="w-full"
            />
            <p className="text-xs text-muted-foreground">
              Number of inference requests that can run simultaneously
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Cache Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Database className="h-5 w-5" />
            Cache Settings
          </CardTitle>
          <CardDescription>
            Model and response caching configuration
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <label className="text-sm font-medium">Enable Cache</label>
              <p className="text-xs text-muted-foreground">
                Cache model outputs for faster subsequent requests
              </p>
            </div>
            <Switch
              checked={settings.enableCache}
              onCheckedChange={(checked) => updateSetting('enableCache', checked)}
            />
          </div>

          {settings.enableCache && (
            <>
              <div className="space-y-2">
                <label className="text-sm font-medium">Cache Directory</label>
                <div className="flex items-center space-x-2">
                  <input
                    type="text"
                    value={settings.cacheDirectory}
                    onChange={(e) => updateSetting('cacheDirectory', e.target.value)}
                    className="flex-1 px-3 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500"
                    placeholder=".cache"
                  />
                  <Button variant="outline" size="sm">
                    <Folder className="h-4 w-4" />
                  </Button>
                </div>
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <label className="text-sm font-medium">Max Cache Size</label>
                  <span className="text-sm text-muted-foreground">{settings.maxCacheSize} MB</span>
                </div>
                <Slider
                  value={[settings.maxCacheSize]}
                  onValueChange={([value]) => updateSetting('maxCacheSize', value)}
                  max={10240}
                  min={100}
                  step={100}
                  className="w-full"
                />
              </div>
            </>
          )}
        </CardContent>
      </Card>

      {/* API Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Globe className="h-5 w-5" />
            API Settings
          </CardTitle>
          <CardDescription>
            REST API and external access configuration
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <label className="text-sm font-medium">Enable REST API</label>
              <p className="text-xs text-muted-foreground">
                Allow external applications to access Inferno via REST API
              </p>
            </div>
            <Switch
              checked={settings.enableRestAPI}
              onCheckedChange={(checked) => updateSetting('enableRestAPI', checked)}
            />
          </div>

          {settings.enableRestAPI && (
            <>
              <div className="space-y-2">
                <label className="text-sm font-medium">API Port</label>
                <input
                  type="number"
                  value={settings.apiPort}
                  onChange={(e) => updateSetting('apiPort', parseInt(e.target.value) || 8080)}
                  className="w-32 px-3 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500"
                  min={1024}
                  max={65535}
                />
                <p className="text-xs text-muted-foreground">
                  Port number for the REST API server
                </p>
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <label className="text-sm font-medium">Enable CORS</label>
                  <p className="text-xs text-muted-foreground">
                    Allow cross-origin requests to the API
                  </p>
                </div>
                <Switch
                  checked={settings.enableCORS}
                  onCheckedChange={(checked) => updateSetting('enableCORS', checked)}
                />
              </div>
            </>
          )}
        </CardContent>
      </Card>

      {/* Security Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Security & Logging
          </CardTitle>
          <CardDescription>
            Authentication, audit, and logging settings
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <label className="text-sm font-medium">Require Authentication</label>
              <p className="text-xs text-muted-foreground">
                Require API keys for external access
              </p>
            </div>
            <Switch
              checked={settings.requireAuthentication}
              onCheckedChange={(checked) => updateSetting('requireAuthentication', checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <label className="text-sm font-medium">Enable Audit Log</label>
              <p className="text-xs text-muted-foreground">
                Log all operations for security and compliance
              </p>
            </div>
            <Switch
              checked={settings.enableAuditLog}
              onCheckedChange={(checked) => updateSetting('enableAuditLog', checked)}
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">Log Level</label>
            <select
              value={settings.logLevel}
              onChange={(e) => updateSetting('logLevel', e.target.value as any)}
              className="w-full px-3 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="error">Error</option>
              <option value="warn">Warning</option>
              <option value="info">Info</option>
              <option value="debug">Debug</option>
            </select>
            <p className="text-xs text-muted-foreground">
              Minimum level of events to log
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Save Status */}
      {!isDirty && (
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-center space-x-2 text-green-600">
              <CheckCircle className="h-4 w-4" />
              <span className="text-sm">All settings are saved</span>
            </div>
          </CardContent>
        </Card>
      )}
      </div>
    </MainLayout>
  );
}