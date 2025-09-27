'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { useSystemInfo } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Users,
  Building,
  Settings,
  Plus,
  MoreHorizontal,
  Activity,
  Database,
  Zap,
  Shield,
  CheckCircle,
  AlertTriangle,
  Edit,
  Trash2,
} from 'lucide-react';

const mockTenants = [
  {
    id: 'tenant_1',
    name: 'Enterprise Corp',
    status: 'active',
    created_at: '2024-01-01T00:00:00Z',
    limits: { max_models: 10, max_concurrent_inferences: 50, storage_quota_gb: 500 },
    usage: { models_count: 7, storage_used_gb: 234, monthly_inferences: 15420 },
    settings: { auto_scaling: true, monitoring_enabled: true }
  },
  {
    id: 'tenant_2',
    name: 'Startup AI',
    status: 'active',
    created_at: '2024-01-15T00:00:00Z',
    limits: { max_models: 5, max_concurrent_inferences: 20, storage_quota_gb: 100 },
    usage: { models_count: 3, storage_used_gb: 45, monthly_inferences: 3240 },
    settings: { auto_scaling: false, monitoring_enabled: true }
  }
];

export default function TenantsPage() {
  const { data: systemInfo, isLoading: systemLoading } = useSystemInfo();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Multi-Tenancy</h1>
          <p className="text-muted-foreground">Manage tenant isolation and resource allocation</p>
        </div>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          New Tenant
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Tenants</CardTitle>
            <Building className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockTenants.length}</div>
            <p className="text-xs text-muted-foreground">organizations</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Models</CardTitle>
            <Database className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {mockTenants.reduce((sum, t) => sum + t.usage.models_count, 0)}
            </div>
            <p className="text-xs text-muted-foreground">across all tenants</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Storage Used</CardTitle>
            <Database className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {mockTenants.reduce((sum, t) => sum + t.usage.storage_used_gb, 0)}GB
            </div>
            <p className="text-xs text-muted-foreground">total allocation</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Isolation</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">100%</div>
            <p className="text-xs text-muted-foreground">secure isolation</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Tenant Management</CardTitle>
          <CardDescription>Monitor and configure tenant resources</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {mockTenants.map((tenant) => (
              <div key={tenant.id} className="border rounded-lg p-4">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-3">
                      <Building className="h-5 w-5 text-primary" />
                      <div>
                        <h3 className="font-semibold">{tenant.name}</h3>
                        <p className="text-sm text-muted-foreground">ID: {tenant.id}</p>
                      </div>
                      <Badge variant={tenant.status === 'active' ? 'default' : 'secondary'}>
                        {tenant.status}
                      </Badge>
                    </div>

                    <div className="grid grid-cols-3 gap-6">
                      <div>
                        <h4 className="text-sm font-medium mb-2">Resource Usage</h4>
                        <div className="space-y-2">
                          <div className="flex items-center justify-between text-sm">
                            <span>Models</span>
                            <span>{tenant.usage.models_count}/{tenant.limits.max_models}</span>
                          </div>
                          <Progress
                            value={(tenant.usage.models_count / tenant.limits.max_models) * 100}
                            className="h-1"
                          />

                          <div className="flex items-center justify-between text-sm">
                            <span>Storage</span>
                            <span>{tenant.usage.storage_used_gb}GB/{tenant.limits.storage_quota_gb}GB</span>
                          </div>
                          <Progress
                            value={(tenant.usage.storage_used_gb / tenant.limits.storage_quota_gb) * 100}
                            className="h-1"
                          />
                        </div>
                      </div>

                      <div>
                        <h4 className="text-sm font-medium mb-2">Monthly Stats</h4>
                        <div className="space-y-1 text-sm">
                          <div className="flex justify-between">
                            <span>Inferences</span>
                            <span className="font-medium">{tenant.usage.monthly_inferences.toLocaleString()}</span>
                          </div>
                          <div className="flex justify-between">
                            <span>Limit</span>
                            <span>{tenant.limits.max_concurrent_inferences}/hr</span>
                          </div>
                        </div>
                      </div>

                      <div>
                        <h4 className="text-sm font-medium mb-2">Settings</h4>
                        <div className="space-y-1 text-sm">
                          <div className="flex justify-between">
                            <span>Auto-scaling</span>
                            <Badge variant={tenant.settings.auto_scaling ? 'default' : 'secondary'}>
                              {tenant.settings.auto_scaling ? 'Enabled' : 'Disabled'}
                            </Badge>
                          </div>
                          <div className="flex justify-between">
                            <span>Monitoring</span>
                            <Badge variant={tenant.settings.monitoring_enabled ? 'default' : 'secondary'}>
                              {tenant.settings.monitoring_enabled ? 'Enabled' : 'Disabled'}
                            </Badge>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 ml-4">
                    <Button variant="outline" size="icon">
                      <Edit className="h-4 w-4" />
                    </Button>
                    <Button variant="outline" size="icon">
                      <Settings className="h-4 w-4" />
                    </Button>
                    <Button variant="outline" size="icon">
                      <MoreHorizontal className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}