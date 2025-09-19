'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { MetricCard } from './metric-card';
import { RecentActivity } from './recent-activity';
import { SystemChart } from './system-chart';
import { ModelStatus } from './model-status';
import { QuickActions } from './quick-actions';
import {
  Brain,
  Cpu,
  HardDrive,
  Activity,
  Clock,
  Users,
  Zap,
  AlertTriangle,
  TrendingUp,
  Play,
} from 'lucide-react';

export function DashboardOverview() {
  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Dashboard Overview</h1>
          <p className="text-muted-foreground">
            Welcome to your Inferno AI/ML platform command center
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant="success" className="flex items-center gap-1">
            <Activity className="h-3 w-3" />
            All Systems Operational
          </Badge>
          <Button>
            <Play className="h-4 w-4 mr-2" />
            Run Inference
          </Button>
        </div>
      </div>

      {/* Key Metrics Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Active Models"
          value="12"
          description="+2 from last week"
          icon={Brain}
          trend="up"
          color="blue"
        />
        <MetricCard
          title="Total Inferences"
          value="1,247"
          description="+18% from yesterday"
          icon={TrendingUp}
          trend="up"
          color="green"
        />
        <MetricCard
          title="Avg Response Time"
          value="247ms"
          description="-12ms from last hour"
          icon={Zap}
          trend="down"
          color="yellow"
        />
        <MetricCard
          title="System Load"
          value="72%"
          description="Normal operation"
          icon={Cpu}
          trend="stable"
          color="blue"
        />
      </div>

      {/* Main Content Grid */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* System Performance Chart */}
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>System Performance</CardTitle>
            <CardDescription>
              Real-time system metrics over the last 24 hours
            </CardDescription>
          </CardHeader>
          <CardContent>
            <SystemChart />
          </CardContent>
        </Card>

        {/* Quick Actions */}
        <Card>
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
            <CardDescription>Common tasks and operations</CardDescription>
          </CardHeader>
          <CardContent>
            <QuickActions />
          </CardContent>
        </Card>
      </div>

      {/* Secondary Content Grid */}
      <div className="grid gap-6 lg:grid-cols-2">
        {/* Model Status */}
        <Card>
          <CardHeader>
            <CardTitle>Model Status</CardTitle>
            <CardDescription>Current status of your AI models</CardDescription>
          </CardHeader>
          <CardContent>
            <ModelStatus />
          </CardContent>
        </Card>

        {/* Recent Activity */}
        <Card>
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
            <CardDescription>Latest system events and operations</CardDescription>
          </CardHeader>
          <CardContent>
            <RecentActivity />
          </CardContent>
        </Card>
      </div>

      {/* System Resource Overview */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
            <Cpu className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">45.2%</div>
            <div className="w-full bg-secondary rounded-full h-2 mt-2">
              <div className="bg-blue-500 h-2 rounded-full w-[45%]"></div>
            </div>
            <p className="text-xs text-muted-foreground mt-2">
              8 cores • 3.2GHz base frequency
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Memory Usage</CardTitle>
            <HardDrive className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">24.1 GB</div>
            <div className="w-full bg-secondary rounded-full h-2 mt-2">
              <div className="bg-green-500 h-2 rounded-full w-[60%]"></div>
            </div>
            <p className="text-xs text-muted-foreground mt-2">
              60% of 40GB • DDR4 3200MHz
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Jobs</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">7</div>
            <div className="flex items-center space-x-2 mt-2">
              <Badge variant="success" className="text-xs">3 Running</Badge>
              <Badge variant="secondary" className="text-xs">4 Queued</Badge>
            </div>
            <p className="text-xs text-muted-foreground mt-2">
              2 batch jobs • 5 inference requests
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}