'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { Input } from '@/components/ui/input';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import {
  useSettings,
  useUpdateSettings,
  useApiKeys,
  useSecurityEvents,
  useSecurityMetrics,
  useCreateApiKey,
  useRevokeApiKey,
  useDeleteApiKey,
  useClearSecurityEvents
} from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Shield,
  Key,
  Lock,
  Eye,
  EyeOff,
  Plus,
  Trash2,
  AlertTriangle,
  CheckCircle,
  Clock,
  User,
  Globe,
  Activity,
  RefreshCw,
  Download,
  Copy,
  Edit,
  X,
} from 'lucide-react';
import { useState } from 'react';
import { ApiKey, SecurityEvent, CreateApiKeyRequest } from '@/types/inferno';
import { toast } from 'react-hot-toast';

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

function formatRelativeTime(timestamp: string): string {
  const now = new Date();
  const time = new Date(timestamp);
  const diffMs = now.getTime() - time.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  return `${diffDays}d ago`;
}

function getEventSeverityColor(severity: string) {
  switch (severity) {
    case 'high':
    case 'critical':
      return 'text-red-600 bg-red-50 border-red-200';
    case 'medium':
      return 'text-yellow-600 bg-yellow-50 border-yellow-200';
    case 'low':
    default:
      return 'text-green-600 bg-green-50 border-green-200';
  }
}

function getEventTypeIcon(eventType: string) {
  switch (eventType) {
    case 'apikeyCreated':
      return Plus;
    case 'apikeyRevoked':
      return X;
    case 'apikeyUsed':
      return CheckCircle;
    case 'authenticationFailed':
      return AlertTriangle;
    case 'unauthorizedAccess':
      return Shield;
    case 'suspiciousActivity':
      return AlertTriangle;
    default:
      return Activity;
  }
}

export default function SecurityPage() {
  const [showApiKeys, setShowApiKeys] = useState(true);
  const [showNewKeyDialog, setShowNewKeyDialog] = useState(false);
  const [newKeyData, setNewKeyData] = useState<CreateApiKeyRequest>({
    name: '',
    permissions: ['read'],
    expires_in_days: 30
  });
  const [createdKey, setCreatedKey] = useState<{ api_key: ApiKey; raw_key: string } | null>(null);

  // API hooks
  const { data: settings, isLoading: settingsLoading } = useSettings();
  const updateSettingsMutation = useUpdateSettings();
  const { data: apiKeys, isLoading: apiKeysLoading } = useApiKeys();
  const { data: securityEvents, isLoading: eventsLoading } = useSecurityEvents(10);
  const { data: securityMetrics, isLoading: metricsLoading } = useSecurityMetrics();
  const createApiKeyMutation = useCreateApiKey();
  const revokeApiKeyMutation = useRevokeApiKey();
  const deleteApiKeyMutation = useDeleteApiKey();
  const clearEventsMutation = useClearSecurityEvents();

  const handleToggleAuth = (enabled: boolean) => {
    if (settings) {
      updateSettingsMutation.mutate({
        ...settings,
        requireAuthentication: enabled
      });
    }
  };

  const handleToggleAuditLog = (enabled: boolean) => {
    if (settings) {
      updateSettingsMutation.mutate({
        ...settings,
        enableAuditLog: enabled
      });
    }
  };

  const handleCopyApiKey = (keyPrefix: string) => {
    navigator.clipboard.writeText(keyPrefix);
    toast.success('API key prefix copied to clipboard');
  };

  const handleRevokeKey = (keyId: string) => {
    if (confirm('Are you sure you want to revoke this API key? This action cannot be undone.')) {
      revokeApiKeyMutation.mutate(keyId);
    }
  };

  const handleDeleteKey = (keyId: string) => {
    if (confirm('Are you sure you want to permanently delete this API key? This action cannot be undone.')) {
      deleteApiKeyMutation.mutate(keyId);
    }
  };

  const handleCreateApiKey = () => {
    if (!newKeyData.name.trim()) {
      toast.error('Please enter a name for the API key');
      return;
    }

    createApiKeyMutation.mutate(newKeyData, {
      onSuccess: (response) => {
        setCreatedKey(response);
        setShowNewKeyDialog(false);
        setNewKeyData({ name: '', permissions: ['read'], expires_in_days: 30 });
      }
    });
  };

  const handleExportSecurityReport = () => {
    const data = {
      timestamp: new Date().toISOString(),
      api_keys: apiKeys?.map(key => ({ ...key, key_hash: undefined })) || [],
      security_events: securityEvents || [],
      metrics: securityMetrics,
      settings: {
        authentication_enabled: settings?.requireAuthentication,
        audit_log_enabled: settings?.enableAuditLog,
      }
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `inferno-security-report-${Date.now()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Security Management</h1>
          <p className="text-muted-foreground">
            Manage API keys, authentication, and security monitoring
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant="outline" className="flex items-center gap-1">
            <Shield className="h-3 w-3" />
            {metricsLoading ? '...' :
             securityMetrics?.failed_auth_attempts_24h ?
             `${securityMetrics.failed_auth_attempts_24h} Alerts` : 'Secure'}
          </Badge>
          <Button variant="outline" onClick={handleExportSecurityReport}>
            <Download className="h-4 w-4 mr-2" />
            Export Report
          </Button>
        </div>
      </div>

      {/* Security Overview */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active API Keys</CardTitle>
            <Key className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {metricsLoading ? <Skeleton className="h-8 w-8" /> : securityMetrics?.active_api_keys || 0}
            </div>
            <p className="text-xs text-muted-foreground">
              {metricsLoading ? '...' :
               `${(securityMetrics?.total_api_keys || 0) - (securityMetrics?.active_api_keys || 0)} revoked`}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Security Events</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {metricsLoading ? <Skeleton className="h-8 w-12" /> : securityMetrics?.security_events_24h || 0}
            </div>
            <p className="text-xs text-muted-foreground">
              Last 24 hours
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Failed Attempts</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">
              {metricsLoading ? <Skeleton className="h-8 w-8" /> : securityMetrics?.failed_auth_attempts_24h || 0}
            </div>
            <p className="text-xs text-muted-foreground">
              Authentication failures
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Auth Status</CardTitle>
            <Lock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {settingsLoading ? (
                <Skeleton className="h-8 w-16" />
              ) : settings?.requireAuthentication ? (
                <span className="text-green-600">Enabled</span>
              ) : (
                <span className="text-red-600">Disabled</span>
              )}
            </div>
            <p className="text-xs text-muted-foreground">
              Authentication required
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Security Settings */}
      <Card>
        <CardHeader>
          <CardTitle>Security Settings</CardTitle>
          <CardDescription>Configure authentication and security policies</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Require Authentication</Label>
              <div className="text-sm text-muted-foreground">
                Require API keys for all requests
              </div>
            </div>
            {settingsLoading ? (
              <Skeleton className="h-6 w-11" />
            ) : (
              <Switch
                checked={settings?.requireAuthentication || false}
                onCheckedChange={handleToggleAuth}
              />
            )}
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Audit Logging</Label>
              <div className="text-sm text-muted-foreground">
                Log all API requests and security events
              </div>
            </div>
            {settingsLoading ? (
              <Skeleton className="h-6 w-11" />
            ) : (
              <Switch
                checked={settings?.enableAuditLog || false}
                onCheckedChange={handleToggleAuditLog}
              />
            )}
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Rate Limiting</Label>
              <div className="text-sm text-muted-foreground">
                Limit requests per API key
              </div>
            </div>
            <Badge variant="secondary">Enabled</Badge>
          </div>
        </CardContent>
      </Card>

      {/* API Key Management */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>API Key Management</CardTitle>
              <CardDescription>Create and manage API keys for system access</CardDescription>
            </div>
            <Button onClick={() => setShowNewKeyDialog(true)}>
              <Plus className="h-4 w-4 mr-2" />
              New API Key
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {apiKeysLoading ? (
            <div className="space-y-4">
              {[...Array(3)].map((_, i) => (
                <Skeleton key={i} className="h-24 w-full" />
              ))}
            </div>
          ) : (
            <div className="space-y-4">
              {apiKeys?.map((apiKey) => (
                <div key={apiKey.id} className="flex items-center justify-between p-4 border rounded-lg">
                  <div className="flex-1">
                    <div className="flex items-center gap-3">
                      <div>
                        <div className="font-medium">{apiKey.name}</div>
                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                          <code className="bg-muted px-2 py-1 rounded text-xs">
                            {showApiKeys ? apiKey.key_prefix + '...' : '••••••••••••••••'}
                          </code>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-4 w-4"
                            onClick={() => handleCopyApiKey(apiKey.key_prefix)}
                          >
                            <Copy className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                      <div className="flex gap-1">
                        {apiKey.permissions.map((permission) => (
                          <Badge key={permission} variant="outline" className="text-xs">
                            {permission}
                          </Badge>
                        ))}
                      </div>
                    </div>
                    <div className="flex items-center gap-4 mt-2 text-sm text-muted-foreground">
                      <span>Created: {formatDate(apiKey.created_at)}</span>
                      {apiKey.last_used && (
                        <span>Last used: {formatRelativeTime(apiKey.last_used)}</span>
                      )}
                      <span>Usage: {apiKey.usage_count.toLocaleString()} requests</span>
                      {apiKey.expires_at && (
                        <span>Expires: {formatDate(apiKey.expires_at)}</span>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant={apiKey.is_active ? 'default' : 'destructive'}>
                      {apiKey.is_active ? 'active' : 'revoked'}
                    </Badge>
                    {apiKey.is_active && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleRevokeKey(apiKey.id)}
                        className="text-yellow-600 hover:text-yellow-700"
                      >
                        <X className="h-4 w-4" />
                      </Button>
                    )}
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleDeleteKey(apiKey.id)}
                      className="text-red-600 hover:text-red-700"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className="flex items-center justify-between pt-4">
            <Button
              variant="ghost"
              onClick={() => setShowApiKeys(!showApiKeys)}
              className="flex items-center gap-2"
            >
              {showApiKeys ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              {showApiKeys ? 'Hide' : 'Show'} API Keys
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Security Events */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Recent Security Events</CardTitle>
              <CardDescription>Monitor authentication attempts and security alerts</CardDescription>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => clearEventsMutation.mutate()}
              disabled={clearEventsMutation.isPending}
            >
              <RefreshCw className="h-4 w-4 mr-2" />
              Clear Events
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {eventsLoading ? (
            <div className="space-y-3">
              {[...Array(5)].map((_, i) => (
                <Skeleton key={i} className="h-16 w-full" />
              ))}
            </div>
          ) : (
            <div className="space-y-3">
              {securityEvents?.map((event) => {
                const EventIcon = getEventTypeIcon(event.event_type);
                const severityClass = getEventSeverityColor(event.severity);

                return (
                  <div key={event.id} className={`flex items-start gap-3 p-3 rounded-lg border ${severityClass}`}>
                    <EventIcon className="h-4 w-4 mt-1" />
                    <div className="flex-1">
                      <div className="flex items-center justify-between">
                        <span className="font-medium capitalize">
                          {event.event_type.replace(/([A-Z])/g, ' $1').toLowerCase()}
                        </span>
                        <Badge variant="outline" className="text-xs">
                          {event.severity}
                        </Badge>
                      </div>
                      <div className="text-sm text-muted-foreground mt-1">
                        {event.source_ip && `IP: ${event.source_ip} • `}
                        {formatRelativeTime(event.timestamp)}
                      </div>
                      <div className="text-sm mt-1">
                        {event.description}
                      </div>
                    </div>
                  </div>
                );
              })}
              {!securityEvents?.length && (
                <div className="text-center py-8 text-muted-foreground">
                  No security events found
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* New API Key Dialog */}
      <Dialog open={showNewKeyDialog} onOpenChange={setShowNewKeyDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create New API Key</DialogTitle>
            <DialogDescription>
              Generate a new API key for accessing the Inferno platform
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="key-name">Key Name</Label>
              <Input
                id="key-name"
                placeholder="e.g., Production App"
                value={newKeyData.name}
                onChange={(e) => setNewKeyData({ ...newKeyData, name: e.target.value })}
              />
            </div>
            <div>
              <Label htmlFor="permissions">Permissions</Label>
              <Select
                value={newKeyData.permissions[0]}
                onValueChange={(value) => setNewKeyData({ ...newKeyData, permissions: [value] })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="read">Read Only</SelectItem>
                  <SelectItem value="write">Read & Write</SelectItem>
                  <SelectItem value="admin">Full Admin</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label htmlFor="expires">Expires In (Days)</Label>
              <Input
                id="expires"
                type="number"
                placeholder="30"
                value={newKeyData.expires_in_days || ''}
                onChange={(e) => setNewKeyData({
                  ...newKeyData,
                  expires_in_days: e.target.value ? parseInt(e.target.value) : undefined
                })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowNewKeyDialog(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleCreateApiKey}
              disabled={createApiKeyMutation.isPending}
            >
              {createApiKeyMutation.isPending ? 'Creating...' : 'Create Key'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Created Key Dialog */}
      <Dialog open={!!createdKey} onOpenChange={() => setCreatedKey(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>API Key Created Successfully</DialogTitle>
            <DialogDescription>
              Copy this key now - it won't be shown again!
            </DialogDescription>
          </DialogHeader>
          {createdKey && (
            <div className="space-y-4">
              <div>
                <Label>API Key</Label>
                <div className="flex items-center gap-2 mt-1">
                  <code className="flex-1 bg-muted p-2 rounded text-sm font-mono">
                    {createdKey.raw_key}
                  </code>
                  <Button
                    size="icon"
                    onClick={() => {
                      navigator.clipboard.writeText(createdKey.raw_key);
                      toast.success('API key copied to clipboard');
                    }}
                  >
                    <Copy className="h-4 w-4" />
                  </Button>
                </div>
              </div>
              <div className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
                <div className="flex items-start gap-2">
                  <AlertTriangle className="h-4 w-4 text-yellow-600 mt-0.5" />
                  <div className="text-sm text-yellow-800">
                    <strong>Important:</strong> Store this key securely. It cannot be recovered once this dialog is closed.
                  </div>
                </div>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button onClick={() => setCreatedKey(null)}>
              I've Saved the Key
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}