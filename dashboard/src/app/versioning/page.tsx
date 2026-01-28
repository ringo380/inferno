'use client';

import { MainLayout } from '@/components/layout/main-layout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { useModels } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';
import {
  GitBranch,
  Tag,
  Clock,
  Download,
  Upload,
  RotateCcw,
  MoreHorizontal,
  CheckCircle,
  AlertTriangle,
  ArrowUpDown,
  History,
  Package,
} from 'lucide-react';

const mockVersions = [
  {
    id: 'v1.2.3',
    model_name: 'llama-7b',
    version: '1.2.3',
    status: 'active',
    created_at: '2024-01-20T10:00:00Z',
    size_mb: 6800,
    accuracy: 92.4,
    deployment_count: 3,
    changelog: 'Improved performance, reduced memory usage'
  },
  {
    id: 'v1.2.2',
    model_name: 'llama-7b',
    version: '1.2.2',
    status: 'deprecated',
    created_at: '2024-01-15T10:00:00Z',
    size_mb: 7200,
    accuracy: 91.8,
    deployment_count: 1,
    changelog: 'Bug fixes and stability improvements'
  },
  {
    id: 'v2.0.1',
    model_name: 'gpt-neo',
    version: '2.0.1',
    status: 'active',
    created_at: '2024-01-18T14:30:00Z',
    size_mb: 2400,
    accuracy: 89.2,
    deployment_count: 2,
    changelog: 'Major architecture updates'
  }
];

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

function formatSize(mb: number): string {
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(1)}GB`;
  }
  return `${mb}MB`;
}

export default function VersioningPage() {
  const { data: models, isLoading: modelsLoading } = useModels();

  const activeVersions = mockVersions.filter(v => v.status === 'active').length;
  const totalDeployments = mockVersions.reduce((sum, v) => sum + v.deployment_count, 0);

  return (
    <MainLayout>
      <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Model Versioning</h1>
          <p className="text-muted-foreground">Manage model versions and deployments</p>
        </div>
        <div className="flex items-center space-x-2">
          <Button variant="outline">
            <Upload className="h-4 w-4 mr-2" />
            Upload Version
          </Button>
          <Button>
            <Package className="h-4 w-4 mr-2" />
            Create Release
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Versions</CardTitle>
            <GitBranch className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{activeVersions}</div>
            <p className="text-xs text-muted-foreground">deployed versions</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Models</CardTitle>
            <Package className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {modelsLoading ? <Skeleton className="h-8 w-8" /> : models?.length || 0}
            </div>
            <p className="text-xs text-muted-foreground">unique models</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Deployments</CardTitle>
            <Upload className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{totalDeployments}</div>
            <p className="text-xs text-muted-foreground">active deployments</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <CheckCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">98.5%</div>
            <p className="text-xs text-muted-foreground">deployment success</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Version History</CardTitle>
          <CardDescription>Track and manage model versions across deployments</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {mockVersions.map((version) => (
              <div key={version.id} className="border rounded-lg p-4">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <Package className="h-5 w-5 text-primary" />
                      <div>
                        <h3 className="font-semibold">{version.model_name} v{version.version}</h3>
                        <p className="text-sm text-muted-foreground">{version.changelog}</p>
                      </div>
                      <Badge variant={version.status === 'active' ? 'default' : 'secondary'}>
                        {version.status}
                      </Badge>
                    </div>

                    <div className="grid grid-cols-5 gap-4 text-sm">
                      <div>
                        <span className="text-muted-foreground">Created:</span>
                        <div className="font-medium">{formatDate(version.created_at)}</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Size:</span>
                        <div className="font-medium">{formatSize(version.size_mb)}</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Accuracy:</span>
                        <div className="font-medium">{version.accuracy}%</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Deployments:</span>
                        <div className="font-medium">{version.deployment_count}</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Tag:</span>
                        <code className="text-xs bg-muted px-1 rounded">v{version.version}</code>
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 ml-4">
                    <Button variant="outline" size="icon" title="Download">
                      <Download className="h-4 w-4" />
                    </Button>
                    <Button variant="outline" size="icon" title="Rollback">
                      <RotateCcw className="h-4 w-4" />
                    </Button>
                    <Button variant="outline" size="icon" title="Compare">
                      <ArrowUpDown className="h-4 w-4" />
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

      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Release Management</CardTitle>
            <CardDescription>Automated release pipelines and rollback strategies</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {[
                { name: 'Canary Deployment', status: 'enabled', description: 'Gradual rollout to production' },
                { name: 'Blue-Green Deploy', status: 'enabled', description: 'Zero-downtime deployments' },
                { name: 'Automatic Rollback', status: 'enabled', description: 'Auto-revert on failures' },
                { name: 'A/B Testing', status: 'disabled', description: 'Compare version performance' }
              ].map((feature) => (
                <div key={feature.name} className="flex items-center justify-between p-3 border rounded-lg">
                  <div>
                    <div className="font-medium">{feature.name}</div>
                    <div className="text-sm text-muted-foreground">{feature.description}</div>
                  </div>
                  <Badge variant={feature.status === 'enabled' ? 'default' : 'secondary'}>
                    {feature.status}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Version Analytics</CardTitle>
            <CardDescription>Performance metrics across versions</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Average Accuracy</span>
                <span className="font-semibold">91.1%</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Model Size Trend</span>
                <span className="font-semibold text-green-600">↓ 5.6%</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Deployment Success</span>
                <span className="font-semibold">98.5%</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Rollback Rate</span>
                <span className="font-semibold">1.5%</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
      </div>
    </MainLayout>
  );
}